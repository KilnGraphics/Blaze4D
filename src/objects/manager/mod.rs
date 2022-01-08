//! Management of vulkan objects.
//!
//! Contains structs and enums to manage creation, access to and destruction of vulkan objects.
//!
//! Access to objects is controlled using synchronization groups. All objects belonging to a
//! synchronization group are accessed as one unit protected by a single timeline semaphore.
//!
//! Allocation and destruction of objects is managed through object sets. A objects set is a
//! collection of objects that have the same lifetime. All objects are created when creating the set
//! and all objects are destroyed only when the entire set is destroyed. All objects of a set
//! belong to the same synchronization group.
//!
//! Both synchronization groups as well as objects sets are managed by smart pointers eliminating
//! the need for manual lifetime management. Object sets keep a reference to their synchronization
//! group internally meaning that if a synchronization group is needed only for a single objects set
//! it suffices to keep the object set alive to also ensure the synchronization group stays alive.
//!
//! Multiple object sets can be accessed in a sequentially consistent manner by using
//! synchronization group sets. This is required to prevent deadlock situations when trying to
//! access multiple sets for the same operation.

pub(super) mod synchronization_group;
pub(super) mod object_set;

mod allocator;
mod worker;

use std::mem::ManuallyDrop;
use std::sync::{Arc, Mutex, MutexGuard};

use ash::vk;

use synchronization_group::*;
use object_set::*;
use crate::objects::manager::allocator::*;
use crate::util::slice_splitter::Splitter;

#[derive(Debug)]
enum ObjectCreateError {
    Vulkan(vk::Result),
    Allocation(AllocationError),
    InvalidReference,
    PoisonedAllocator,
}

impl<'s> From<ash::vk::Result> for ObjectCreateError {
    fn from(err: vk::Result) -> Self {
        ObjectCreateError::Vulkan(err)
    }
}

impl<'s> From<AllocationError> for ObjectCreateError {
    fn from(err: AllocationError) -> Self {
        ObjectCreateError::Allocation(err)
    }
}

// Internal struct used during object creation
enum TemporaryObjectData<'a> {
    Buffer{
        handle: vk::Buffer,
        allocation: Option<Allocation>,
        desc: &'a BufferRequestDescription,
    },
    BufferView{
        handle: vk::BufferView,
        desc: &'a BufferViewRequestDescription,
    },
    Image{
        handle: vk::Image,
        allocation: Option<Allocation>,
        desc: &'a ImageRequestDescription,
    },
    ImageView{
        handle: vk::ImageView,
        desc: &'a ImageViewRequestDescription,
    }
}

impl<'a> TemporaryObjectData<'a> {
    fn make_buffer(desc: &'a BufferRequestDescription) -> Self {
        Self::Buffer {
            handle: vk::Buffer::null(),
            allocation: None,
            desc
        }
    }

    fn make_buffer_view(desc: &'a BufferViewRequestDescription) -> Self {
        Self::BufferView {
            handle: vk::BufferView::null(),
            desc
        }
    }

    fn make_image(desc: &'a ImageRequestDescription) -> Self {
        Self::Image {
            handle: vk::Image::null(),
            allocation: None,
            desc
        }
    }

    fn make_image_view(desc: &'a ImageViewRequestDescription) -> Self {
        Self::ImageView {
            handle: vk::ImageView::null(),
            desc
        }
    }
}

// Internal implementation of the object manager
struct ObjectManagerImpl {
    device: crate::rosella::DeviceContext,
    allocator: Allocator,
}

impl ObjectManagerImpl {
    fn new(device: crate::rosella::DeviceContext) -> Self {
        let allocator = Allocator::new(device.clone());

        Self{
            device,
            allocator,
        }
    }

    /// Creates a timeline semaphore for use in a synchronization group
    fn create_timeline_semaphore(&self, initial_value: u64) -> vk::Semaphore {
        let mut timeline_info = vk::SemaphoreTypeCreateInfo::builder()
            .semaphore_type(vk::SemaphoreType::TIMELINE)
            .initial_value(initial_value);
        let info = vk::SemaphoreCreateInfo::builder().push_next(&mut timeline_info);

        unsafe {
            self.device.vk().create_semaphore(&info.build(), None).unwrap()
        }
    }

    /// Destroys a semaphore previously created using [`ObjectManagerImpl::create_timeline_semaphore`]
    fn destroy_semaphore(&self, semaphore: vk::Semaphore) {
        unsafe {
            self.device.vk().destroy_semaphore(semaphore, None)
        }
    }

    /// Destroys a set of temporary objects. This is used if an error is encountered during the
    /// build process.
    fn destroy_temporary_objects(&self, objects: &mut [TemporaryObjectData]) {
        // First destroy any object that might have a dependency on other objects
        for object in objects.iter() {
            match object {
                TemporaryObjectData::BufferView { handle, .. } => {
                    if *handle != vk::BufferView::null() {
                        unsafe { self.device.vk().destroy_buffer_view(*handle, None) }
                    }
                },
                TemporaryObjectData::ImageView { handle, .. } => {
                    if *handle != vk::ImageView::null() {
                        unsafe { self.device.vk().destroy_image_view(*handle, None) }
                    }
                },
                _ => {}
            }
        };
        // Then destroy everything else
        for object in objects.iter_mut() {
            match object {
                TemporaryObjectData::Buffer { handle, allocation, .. } => {
                    if *handle != vk::Buffer::null() {
                        unsafe { self.device.vk().destroy_buffer(*handle, None) }
                    }
                    allocation.take().map(|alloc| self.allocator.free(alloc));
                },
                TemporaryObjectData::Image { handle, allocation,  .. } => {
                    if *handle != vk::Image::null() {
                        unsafe { self.device.vk().destroy_image(*handle, None) }
                    }
                    allocation.take().map(|alloc| self.allocator.free(alloc));
                },
                _ => {}
            }
        }
    }

    /// Creates the objects for a temporary object data list
    fn create_temporary_objects(&self, objects: &mut [TemporaryObjectData]) -> Result<(), ObjectCreateError> {
        // Create all objects that do not depend on other objects
        for object in objects.iter_mut() {
            match object {
                TemporaryObjectData::Buffer {
                    handle,
                    desc,
                    ..
                } => {
                    if handle == &vk::Buffer::null() {
                        let create_info = vk::BufferCreateInfo::builder()
                            .size(desc.description.size)
                            .usage(desc.description.usage_flags)
                            .sharing_mode(vk::SharingMode::EXCLUSIVE);

                        *handle = unsafe {
                            self.device.vk().create_buffer(&create_info.build(), None)
                        }?;
                    }
                },
                TemporaryObjectData::Image {
                    handle,
                    desc,
                    ..
                } => {
                    if handle == &vk::Image::null() {
                        let create_info = vk::ImageCreateInfo::builder()
                            .image_type(desc.description.spec.size.get_vulkan_type())
                            .format(desc.description.spec.format.get_format())
                            .extent(desc.description.spec.size.as_extent_3d())
                            .mip_levels(desc.description.spec.size.get_mip_levels())
                            .array_layers(desc.description.spec.size.get_array_layers())
                            .samples(desc.description.spec.sample_count)
                            .tiling(vk::ImageTiling::OPTIMAL)
                            .usage(desc.description.usage_flags)
                            .sharing_mode(vk::SharingMode::EXCLUSIVE);

                        *handle = unsafe {
                            self.device.vk().create_image(&create_info.build(), None)
                        }?;
                    }
                },
                _ => {}
            }
        }

        // Allocate memory for objects
        {
            for object in objects.iter_mut() {
                match object {
                    TemporaryObjectData::Buffer {
                        handle,
                        allocation,
                        desc
                    } => {
                        if allocation.is_none() {
                            *allocation = Some(self.allocator.allocate_buffer_memory(*handle, &desc.strategy)?);
                            let alloc = allocation.as_ref().unwrap();

                            unsafe {
                                self.device.vk().bind_buffer_memory(*handle, alloc.memory(), alloc.offset())
                            }?;
                        }
                    }
                    TemporaryObjectData::Image {
                        handle,
                        allocation,
                        desc
                    } => {
                        if allocation.is_none() {
                            *allocation = Some(self.allocator.allocate_image_memory(*handle, &desc.strategy)?);
                            let alloc = allocation.as_ref().unwrap();

                            unsafe {
                                self.device.vk().bind_image_memory(*handle, alloc.memory(), alloc.offset())
                            }?;
                        }
                    }
                    _ => {}
                }
            }
        }

        // Create dependant objects
        for i in 0..objects.len() {
            let (split, elem) = Splitter::new(objects, i);

            match elem {
                TemporaryObjectData::BufferView {
                    handle,
                    desc
                } => {
                    if handle != &vk::BufferView::null() {
                        let buffer = match desc.owning_set.as_ref() {
                            Some(set) => {
                                set.get_buffer_handle(desc.buffer_id).ok_or(ObjectCreateError::InvalidReference)?
                            }
                            None => {
                                let index = desc.buffer_id.get_index() as usize;
                                match split.get(index).ok_or(ObjectCreateError::InvalidReference)? {
                                    TemporaryObjectData::Buffer { handle, .. } => *handle,
                                    _ => return Err(ObjectCreateError::InvalidReference)
                                }
                            }
                        };

                        let create_info = vk::BufferViewCreateInfo::builder()
                            .buffer(buffer)
                            .format(desc.description.format.get_format())
                            .offset(desc.description.range.offset)
                            .range(desc.description.range.length);

                        *handle = unsafe {
                            self.device.vk().create_buffer_view(&create_info.build(), None)?
                        }
                    }
                }
                TemporaryObjectData::ImageView {
                    handle,
                    desc
                } => {
                    if handle != &vk::ImageView::null() {
                        let image = match desc.owning_set.as_ref() {
                            Some(set) => {
                                set.get_image_handle(desc.image_id).ok_or(ObjectCreateError::InvalidReference)?
                            }
                            None => {
                                let index = desc.image_id.get_index() as usize;
                                match split.get(index).ok_or(ObjectCreateError::InvalidReference)? {
                                    TemporaryObjectData::Image { handle, .. } => *handle,
                                    _ => return Err(ObjectCreateError::InvalidReference)
                                }
                            }
                        };

                        let create_info = vk::ImageViewCreateInfo::builder()
                            .image(image)
                            .view_type(desc.description.view_type)
                            .format(desc.description.format.get_format())
                            .components(desc.description.components)
                            .subresource_range(desc.description.subresource_range.as_vk_subresource_range());

                        *handle = unsafe {
                            self.device.vk().create_image_view(&create_info, None)?
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Converts a object request description list to a temporary object data list
    fn create_temporary_object_data<'a>(&self, objects: &'a [ObjectRequestDescription]) -> Vec<TemporaryObjectData<'a>> {
        objects.iter().map(|request| {
            match request {
                ObjectRequestDescription::Buffer(desc) => {
                    TemporaryObjectData::make_buffer(desc)
                }
                ObjectRequestDescription::BufferView(desc) => {
                    TemporaryObjectData::make_buffer_view(desc)
                }
                ObjectRequestDescription::Image(desc) => {
                    TemporaryObjectData::make_image(desc)
                }
                ObjectRequestDescription::ImageView(desc) => {
                    TemporaryObjectData::make_image_view(desc)
                }
            }
        }).collect()
    }

    /// Converts a temporary object data list to a object data list and allocation meta instance
    fn flatten_temporary_object_data(&self, objects: Vec<TemporaryObjectData>) -> (Box<[ObjectData]>, Box<[Allocation]>) {
        let mut allocations = Vec::new();
        let mut object_data = Vec::with_capacity(objects.len());

        for object in objects.into_iter() {
            object_data.push(match object {
                TemporaryObjectData::Buffer { handle, allocation, .. } => {
                    match allocation {
                        None => {}
                        Some(allocation) => allocations.push(allocation)
                    }
                    ObjectData::Buffer { handle }
                }
                TemporaryObjectData::BufferView { handle, desc, .. } => {
                    ObjectData::BufferView {
                        handle,
                        source_set: desc.owning_set.clone(),
                    }
                }
                TemporaryObjectData::Image { handle, allocation, .. } => {
                    match allocation {
                        None => {}
                        Some(allocation) => allocations.push(allocation)
                    }
                    ObjectData::Image { handle }
                }
                TemporaryObjectData::ImageView { handle, desc, .. } => {
                    ObjectData::ImageView {
                        handle,
                        source_set: desc.owning_set.clone(),
                    }
                }
            });
        }

        (object_data.into_boxed_slice(), allocations.into_boxed_slice())
    }

    /// Creates objects for a object request description list
    fn create_objects(&self, objects: &[ObjectRequestDescription]) -> (Box<[ObjectData]>, Box<[Allocation]>) {
        let mut objects = self.create_temporary_object_data(objects);
        self.create_temporary_objects(objects.as_mut_slice()).map_err(|err| {
            self.destroy_temporary_objects(objects.as_mut_slice()); err
        }).unwrap();

        self.flatten_temporary_object_data(objects)
    }

    /// Destroys objects previously created using [`ObjectManagerImpl::create_objects`]
    fn destroy_objects(&self, objects: &[ObjectData], allocations: Box<[Allocation]>) {
        for object in objects {
            match object {
                ObjectData::BufferView { handle, .. } => {
                    unsafe{ self.device.vk().destroy_buffer_view(*handle, None) }
                }
                ObjectData::ImageView { handle, .. } => {
                    unsafe{ self.device.vk().destroy_image_view(*handle, None) }
                }
                _ => {}
            }
        }
        for object in objects {
            match object {
                ObjectData::Buffer { handle, .. } => {
                    unsafe{ self.device.vk().destroy_buffer(*handle, None) }
                }
                ObjectData::Image { handle, .. } => {
                    unsafe{ self.device.vk().destroy_image(*handle, None) }
                }
                _ => {}
            }
        }

        for allocation in allocations.into_vec() {
            self.allocator.free(allocation);
        }
    }
}

/// Public object manager api.
///
/// This is a smart pointer reference to an internal struct.
pub struct ObjectManager(Arc<ObjectManagerImpl>);

impl ObjectManager {
    /// Creates a new ObjectManager
    pub fn new(device: crate::rosella::DeviceContext) -> Self {
        Self(Arc::new(ObjectManagerImpl::new(device)))
    }

    /// Creates a new synchronization group managed by this object manager
    pub fn create_synchronization_group(&self) -> SynchronizationGroup {
        SynchronizationGroup::new(self.clone(), self.0.create_timeline_semaphore(0u64))
    }

    /// Creates a new object set builder
    pub fn create_object_set(&self, synchronization_group: SynchronizationGroup) -> ObjectSetBuilder {
        // if synchronization_group.get_manager() != self {
        //     panic!("Synchronization group is not owned by manager")
        // } TODO fix pointer equality

        ObjectSetBuilder::new(synchronization_group)
    }

    /// Creates a new object set builder without a synchronization group
    pub fn create_no_group_object_set(&self) -> ObjectSetBuilder {
        ObjectSetBuilder::new_no_group(self.clone())
    }

    // Internal function that destroys a semaphore created for a synchronization group
    fn destroy_semaphore(&self, semaphore: vk::Semaphore) {
        self.0.destroy_semaphore(semaphore)
    }

    fn create_objects(&self, objects: &[ObjectRequestDescription]) -> (Box<[ObjectData]>, Box<[Allocation]>) {
        self.0.create_objects(objects)
    }

    // Internal function that destroys objects and allocations created for a object set
    fn destroy_objects(&self, objects: Box<[ObjectData]>, allocations: Box<[Allocation]>) {
        self.0.destroy_objects(&objects, allocations)
    }
}

impl Clone for ObjectManager {
    fn clone(&self) -> Self {
        Self( self.0.clone() )
    }
}

#[cfg(test)]
mod tests {
    use crate::objects::{BufferRange, ImageSize, ImageSpec};
    use crate::objects::buffer::{BufferCreateDesc, BufferViewCreateDesc};
    use crate::objects::image::ImageCreateDesc;
    use super::*;

    fn create() -> ObjectManager {
        let (_, device) = crate::test::make_headless_instance_device();
        ObjectManager::new(device)
    }

    #[test]
    fn create_destroy() {
        let (_, device) = crate::test::make_headless_instance_device();
        let manager = ObjectManager::new(device);
        drop(manager);
    }

    #[test]
    fn create_synchronization_group() {
        let manager = create();
        let group = manager.create_synchronization_group();
        let group2 = manager.create_synchronization_group();

        assert_eq!(group, group);
        assert_eq!(group2, group2);
        assert_ne!(group, group2);

        drop(group2);
        drop(group);
    }

    #[test]
    fn create_object_set_buffer() {
        let manager = create();
        let group = manager.create_synchronization_group();

        let mut builder = manager.create_object_set(group.clone());
        let desc = BufferCreateDesc::new_simple(1024, vk::BufferUsageFlags::TRANSFER_SRC | vk::BufferUsageFlags::TRANSFER_DST);
        let id = builder.add_default_gpu_only_buffer(desc);

        let set = builder.build();

        assert_eq!(set.get_synchronization_group(), Some(&group));

        assert!(set.get_buffer_handle(id).is_some());

        drop(set);
    }

    #[test]
    fn create_object_set_image() {
        let manager = create();
        let group = manager.create_synchronization_group();

        let mut builder = manager.create_object_set(group.clone());
        let desc = ImageCreateDesc::new_simple(ImageSpec::new_single_sample(ImageSize::make_1d(32), &crate::objects::Format::R16_UNORM),
            vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::TRANSFER_DST);
        let id = builder.add_default_gpu_only_image(desc);

        let set = builder.build();

        assert_eq!(set.get_synchronization_group(), Some(&group));

        assert!(set.get_image_handle(id).is_some());

        drop(set);
    }

    #[test]
    fn create_object_set_buffer_view() {
        let manager = create();
        let group = manager.create_synchronization_group();

        let mut builder = manager.create_object_set(group.clone());
        let buffer_desc = BufferCreateDesc::new_simple(
            1024,
            vk::BufferUsageFlags::TRANSFER_SRC | vk::BufferUsageFlags::TRANSFER_DST);
        let buffer_id = builder.add_default_gpu_only_buffer(buffer_desc);
        let view_desc = BufferViewCreateDesc::new_simple(BufferRange { offset: 256, length: 256 }, &crate::objects::Format::R16_UNORM);
        let view_id = builder.add_internal_buffer_view(view_desc, buffer_id);

        let set = builder.build();

        assert!(set.get_buffer_handle(buffer_id).is_some());
        assert!(set.get_buffer_view_handle(view_id).is_some());

        let mut builder = manager.create_object_set(group.clone());
        let view_desc = BufferViewCreateDesc::new_simple(BufferRange { offset: 256, length: 256 }, &crate::objects::Format::R16_UNORM);
        let view2_id = builder.add_external_buffer_view(view_desc, set.clone(), buffer_id);

        let set2 = builder.build();

        assert!(set2.get_buffer_view_handle(view2_id).is_some());

        // Test that original set does not get destroyed early
        drop(set);
        drop(set2);
    }
}