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

use std::sync::Arc;

use ash::vk;

use synchronization_group::*;
use object_set::*;
use crate::objects::buffer::{BufferCreateDesc, BufferViewCreateDesc};
use crate::objects::id;
use crate::objects::image::{ImageCreateDesc, ImageViewCreateDesc};
use crate::objects::manager::allocator::*;
use crate::util::slice_splitter::Splitter;

#[derive(Debug)]
enum ObjectCreateError {
    Vulkan(vk::Result),
    Allocation(AllocationError),
    InvalidReference,
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

struct BufferCreateMetadata<'a> {
    handle: vk::Buffer,
    allocation: Option<Allocation>,
    desc: &'a BufferRequestDescription,
}

struct BufferViewCreateMetadata<'a> {
    handle: vk::BufferView,
    desc: &'a BufferViewRequestDescription,
}

struct ImageCreateMetadata<'a> {
    handle: vk::Image,
    allocation: Option<Allocation>,
    desc: &'a ImageRequestDescription,
}

struct ImageViewCreateMetadata<'a> {
    handle: vk::ImageView,
    desc: &'a ImageViewRequestDescription,
}

struct BinarySemaphoreCreateMetadata<'a> {
    handle: vk::Semaphore,
    #[allow(unused)] // Nothing in it thus far but well add it for completeness sake
    desc: &'a BinarySemaphoreRequestDescription,
}

struct TimelineSemaphoreCreateMetadata<'a> {
    handle: vk::Semaphore,
    desc: &'a TimelineSemaphoreRequestDescription
}

struct EventCreateMetadata<'a> {
    handle: vk::Event,
    desc: &'a EventRequestDescription,
}

struct FenceCreateMetadata<'a> {
    handle: vk::Fence,
    desc: &'a FenceRequestDescription,
}

/// Internal struct used during object creation
enum ObjectCreateMetadata<'a> {
    Buffer(BufferCreateMetadata<'a>),
    BufferView(BufferViewCreateMetadata<'a>),
    Image(ImageCreateMetadata<'a>),
    ImageView(ImageViewCreateMetadata<'a>),
    BinarySemaphore(BinarySemaphoreCreateMetadata<'a>),
    TimelineSemaphore(TimelineSemaphoreCreateMetadata<'a>),
    Event(EventCreateMetadata<'a>),
    Fence(FenceCreateMetadata<'a>),
}

impl<'a> ObjectCreateMetadata<'a> {
    fn make_buffer(desc: &'a BufferRequestDescription) -> Self {
        Self::Buffer(BufferCreateMetadata{
            handle: vk::Buffer::null(),
            allocation: None,
            desc
        })
    }

    fn make_buffer_view(desc: &'a BufferViewRequestDescription) -> Self {
        Self::BufferView(BufferViewCreateMetadata{
            handle: vk::BufferView::null(),
            desc
        })
    }

    fn make_image(desc: &'a ImageRequestDescription) -> Self {
        Self::Image(ImageCreateMetadata{
            handle: vk::Image::null(),
            allocation: None,
            desc
        })
    }

    fn make_image_view(desc: &'a ImageViewRequestDescription) -> Self {
        Self::ImageView(ImageViewCreateMetadata{
            handle: vk::ImageView::null(),
            desc
        })
    }

    fn make_binary_semaphore(desc: &'a BinarySemaphoreRequestDescription) -> Self {
        Self::BinarySemaphore(BinarySemaphoreCreateMetadata{
            handle: vk::Semaphore::null(),
            desc
        })
    }

    fn make_timeline_semaphore(desc: &'a TimelineSemaphoreRequestDescription) -> Self {
        Self::TimelineSemaphore(TimelineSemaphoreCreateMetadata{
            handle: vk::Semaphore::null(),
            desc
        })
    }

    fn make_event(desc: &'a EventRequestDescription) -> Self {
        Self::Event(EventCreateMetadata{
            handle: vk::Event::null(),
            desc
        })
    }

    fn make_fence(desc: &'a FenceRequestDescription) -> Self {
        Self::Fence(FenceCreateMetadata{
            handle: vk::Fence::null(),
            desc
        })
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
    fn create_group_semaphore(&self, initial_value: u64) -> vk::Semaphore {
        let mut timeline_info = vk::SemaphoreTypeCreateInfo::builder()
            .semaphore_type(vk::SemaphoreType::TIMELINE)
            .initial_value(initial_value);
        let info = vk::SemaphoreCreateInfo::builder().push_next(&mut timeline_info);

        unsafe {
            self.device.vk().create_semaphore(&info.build(), None).unwrap()
        }
    }

    /// Destroys a semaphore previously created using [`ObjectManagerImpl::create_timeline_semaphore`]
    fn destroy_group_semaphore(&self, semaphore: vk::Semaphore) {
        unsafe {
            self.device.vk().destroy_semaphore(semaphore, None)
        }
    }

    /// Destroys a set of temporary objects. This is used if an error is encountered during the
    /// build process.
    fn destroy_temporary_objects(&self, objects: &mut [ObjectCreateMetadata]) {
        // Iterate in reverse order to respect dependencies
        for object in objects.iter_mut().rev() {
            match object {
                ObjectCreateMetadata::Buffer(BufferCreateMetadata{ handle, allocation, .. }) => {
                    if *handle != vk::Buffer::null() {
                        unsafe { self.device.vk().destroy_buffer(*handle, None) }
                    }
                    allocation.take().map(|alloc| self.allocator.free(alloc));
                },
                ObjectCreateMetadata::BufferView(BufferViewCreateMetadata{ handle, .. }) => {
                    if *handle != vk::BufferView::null() {
                        unsafe { self.device.vk().destroy_buffer_view(*handle, None) }
                    }
                },
                ObjectCreateMetadata::Image(ImageCreateMetadata{ handle, allocation, .. }) => {
                    if *handle != vk::Image::null() {
                        unsafe { self.device.vk().destroy_image(*handle, None) }
                    }
                    allocation.take().map(|alloc| self.allocator.free(alloc));
                },
                ObjectCreateMetadata::ImageView(ImageViewCreateMetadata{ handle, .. }) => {
                    if *handle != vk::ImageView::null() {
                        unsafe { self.device.vk().destroy_image_view(*handle, None) }
                    }
                },
                ObjectCreateMetadata::BinarySemaphore(BinarySemaphoreCreateMetadata{ handle, .. }) => {
                    if *handle != vk::Semaphore::null() {
                        unsafe { self.device.vk().destroy_semaphore(*handle, None) }
                    }
                },
                ObjectCreateMetadata::TimelineSemaphore(TimelineSemaphoreCreateMetadata{ handle, .. }) => {
                    if *handle != vk::Semaphore::null() {
                        unsafe { self.device.vk().destroy_semaphore(*handle, None) }
                    }
                },
                ObjectCreateMetadata::Event(EventCreateMetadata{ handle, .. }) => {
                    if *handle != vk::Event::null() {
                        unsafe { self.device.vk().destroy_event(*handle, None) }
                    }
                },
                ObjectCreateMetadata::Fence(FenceCreateMetadata{ handle, .. }) => {
                    if *handle != vk::Fence::null() {
                        unsafe { self.device.vk().destroy_fence(*handle, None) }
                    }
                }
            }
        }
    }

    fn create_buffer(&self, meta: &mut BufferCreateMetadata) -> Result<(), ObjectCreateError> {
        if meta.handle == vk::Buffer::null() {
            let create_info = vk::BufferCreateInfo::builder()
                .size(meta.desc.description.size)
                .usage(meta.desc.description.usage_flags)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            meta.handle = unsafe {
                self.device.vk().create_buffer(&create_info.build(), None)
            }?;
        }
        if meta.allocation.is_none() {
            meta.allocation = Some(self.allocator.allocate_buffer_memory(meta.handle, &meta.desc.strategy)?);
            let alloc = meta.allocation.as_ref().unwrap();

            unsafe {
                self.device.vk().bind_buffer_memory(meta.handle, alloc.memory(), alloc.offset())
            }?;
        }
        Ok(())
    }

    fn create_buffer_view(&self, meta: &mut BufferViewCreateMetadata, split: &Splitter<ObjectCreateMetadata>) -> Result<(), ObjectCreateError> {
        if meta.handle == vk::BufferView::null() {
            let buffer = match meta.desc.owning_set.as_ref() {
                Some(set) => {
                    set.get_buffer_handle(meta.desc.buffer_id).ok_or(ObjectCreateError::InvalidReference)?
                }
                None => {
                    let index = meta.desc.buffer_id.get_index() as usize;
                    match split.get(index).ok_or(ObjectCreateError::InvalidReference)? {
                        ObjectCreateMetadata::Buffer(BufferCreateMetadata{ handle, .. }) => *handle,
                        _ => return Err(ObjectCreateError::InvalidReference)
                    }
                }
            };

            let create_info = vk::BufferViewCreateInfo::builder()
                .buffer(buffer)
                .format(meta.desc.description.format.get_format())
                .offset(meta.desc.description.range.offset)
                .range(meta.desc.description.range.length);

            meta.handle = unsafe {
                self.device.vk().create_buffer_view(&create_info.build(), None)?
            }
        }
        Ok(())
    }

    fn create_image(&self, meta: &mut ImageCreateMetadata) -> Result<(), ObjectCreateError> {
        if meta.handle == vk::Image::null() {
            let create_info = vk::ImageCreateInfo::builder()
                .image_type(meta.desc.description.spec.size.get_vulkan_type())
                .format(meta.desc.description.spec.format.get_format())
                .extent(meta.desc.description.spec.size.as_extent_3d())
                .mip_levels(meta.desc.description.spec.size.get_mip_levels())
                .array_layers(meta.desc.description.spec.size.get_array_layers())
                .samples(meta.desc.description.spec.sample_count)
                .tiling(vk::ImageTiling::OPTIMAL) // TODO we need some way to turn this linear
                .usage(meta.desc.description.usage_flags)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            meta.handle = unsafe {
                self.device.vk().create_image(&create_info.build(), None)
            }?;
        }
        if meta.allocation.is_none() {
            meta.allocation = Some(self.allocator.allocate_image_memory(meta.handle, &meta.desc.strategy)?);
            let alloc = meta.allocation.as_ref().unwrap();

            unsafe {
                self.device.vk().bind_image_memory(meta.handle, alloc.memory(), alloc.offset())
            }?;
        }
        Ok(())
    }

    fn create_image_view(&self, meta: &mut ImageViewCreateMetadata, split: Splitter<ObjectCreateMetadata>) -> Result<(), ObjectCreateError> {
        if meta.handle == vk::ImageView::null() {
            let image = match meta.desc.owning_set.as_ref() {
                Some(set) => {
                    set.get_image_handle(meta.desc.image_id).ok_or(ObjectCreateError::InvalidReference)?
                }
                None => {
                    let index = meta.desc.image_id.get_index() as usize;
                    match split.get(index).ok_or(ObjectCreateError::InvalidReference)? {
                        ObjectCreateMetadata::Image(ImageCreateMetadata{ handle, .. }) => *handle,
                        _ => return Err(ObjectCreateError::InvalidReference)
                    }
                }
            };

            let create_info = vk::ImageViewCreateInfo::builder()
                .image(image)
                .view_type(meta.desc.description.view_type)
                .format(meta.desc.description.format.get_format())
                .components(meta.desc.description.components)
                .subresource_range(meta.desc.description.subresource_range.as_vk_subresource_range());

            meta.handle = unsafe {
                self.device.vk().create_image_view(&create_info, None)?
            }
        }
        Ok(())
    }

    fn create_binary_semaphore(&self, meta: &mut BinarySemaphoreCreateMetadata) -> Result<(), ObjectCreateError> {
        if meta.handle == vk::Semaphore::null() {
            let create_info = vk::SemaphoreCreateInfo::builder();

            meta.handle = unsafe {
                self.device.vk().create_semaphore(&create_info, None)?
            }
        }
        Ok(())
    }

    fn create_timeline_semaphore(&self, meta: &mut TimelineSemaphoreCreateMetadata) -> Result<(), ObjectCreateError> {
        if meta.handle == vk::Semaphore::null() {
            let mut timeline_info = vk::SemaphoreTypeCreateInfo::builder()
                .semaphore_type(vk::SemaphoreType::TIMELINE)
                .initial_value(meta.desc.initial_value);

            let create_info = vk::SemaphoreCreateInfo::builder()
                .push_next(&mut timeline_info);

            meta.handle = unsafe {
                self.device.vk().create_semaphore(&create_info, None)?
            }
        }
        Ok(())
    }

    fn create_event(&self, meta: &mut EventCreateMetadata) -> Result<(), ObjectCreateError> {
        if meta.handle == vk::Event::null() {
            let flags = if meta.desc.device_only {
                vk::EventCreateFlags::DEVICE_ONLY_KHR
            } else {
                vk::EventCreateFlags::empty()
            };

            let create_info = vk::EventCreateInfo::builder()
                .flags(flags);

            meta.handle = unsafe {
                self.device.vk().create_event(&create_info, None)?
            }
        }
        Ok(())
    }

    fn create_fence(&self, meta: &mut FenceCreateMetadata) -> Result<(), ObjectCreateError> {
        if meta.handle == vk::Fence::null() {
            let flags = if meta.desc.signaled {
                vk::FenceCreateFlags::SIGNALED
            } else {
                vk::FenceCreateFlags::empty()
            };

            let create_info = vk::FenceCreateInfo::builder()
                .flags(flags);

            meta.handle = unsafe {
                self.device.vk().create_fence(&create_info, None)?
            }
        }
        Ok(())
    }

    /// Creates the objects for a temporary object data list
    fn create_objects_for_metadata(&self, objects: &mut [ObjectCreateMetadata]) -> Result<(), ObjectCreateError> {

        // Since every entry can only reference previous entries its safe to iterate over them just once
        for i in 0..objects.len() {
            let (split, object) = Splitter::new(objects, i);

            match object {
                ObjectCreateMetadata::Buffer(meta) => self.create_buffer(meta)?,
                ObjectCreateMetadata::BufferView(meta) => self.create_buffer_view(meta, &split)?,
                ObjectCreateMetadata::Image(meta) => self.create_image(meta)?,
                ObjectCreateMetadata::ImageView(meta) => self.create_image_view(meta, split)?,
                ObjectCreateMetadata::BinarySemaphore(meta) => self.create_binary_semaphore(meta)?,
                ObjectCreateMetadata::TimelineSemaphore(meta) => self.create_timeline_semaphore(meta)?,
                ObjectCreateMetadata::Event(meta) => self.create_event(meta)?,
                ObjectCreateMetadata::Fence(meta) => self.create_fence(meta)?,
            }
        }

        Ok(())
    }

    /// Converts a object request description list to a temporary object data list
    fn generate_objects_metadata<'a>(&self, objects: &'a [ObjectRequestDescription]) -> Vec<ObjectCreateMetadata<'a>> {
        objects.iter().map(|request| {
            match request {
                ObjectRequestDescription::Buffer(desc) => {
                    ObjectCreateMetadata::make_buffer(desc)
                }
                ObjectRequestDescription::BufferView(desc) => {
                    ObjectCreateMetadata::make_buffer_view(desc)
                }
                ObjectRequestDescription::Image(desc) => {
                    ObjectCreateMetadata::make_image(desc)
                }
                ObjectRequestDescription::ImageView(desc) => {
                    ObjectCreateMetadata::make_image_view(desc)
                }
                ObjectRequestDescription::BinarySemaphore(desc) => {
                    ObjectCreateMetadata::make_binary_semaphore(desc)
                }
                ObjectRequestDescription::TimelineSemaphore(desc) => {
                    ObjectCreateMetadata::make_timeline_semaphore(desc)
                }
                ObjectRequestDescription::Event(desc) => {
                    ObjectCreateMetadata::make_event(desc)
                }
                ObjectRequestDescription::Fence(desc) => {
                    ObjectCreateMetadata::make_fence(desc)
                }
            }
        }).collect()
    }

    /// Converts a temporary object data list to a object data list and allocation meta instance
    fn flatten_object_metadata(&self, objects: Vec<ObjectCreateMetadata>) -> (Box<[ObjectData]>, Box<[Allocation]>) {
        let mut allocations = Vec::new();
        let mut object_data = Vec::with_capacity(objects.len());

        for object in objects.into_iter() {
            object_data.push(match object {
                ObjectCreateMetadata::Buffer(BufferCreateMetadata{ handle, allocation, .. }) => {
                    match allocation {
                        None => {}
                        Some(allocation) => allocations.push(allocation)
                    }
                    ObjectData::Buffer { handle }
                }
                ObjectCreateMetadata::BufferView(BufferViewCreateMetadata{ handle, desc, .. }) => {
                    ObjectData::BufferView {
                        handle,
                        source_set: desc.owning_set.clone(),
                    }
                }
                ObjectCreateMetadata::Image(ImageCreateMetadata{ handle, allocation, .. }) => {
                    match allocation {
                        None => {}
                        Some(allocation) => allocations.push(allocation)
                    }
                    ObjectData::Image { handle }
                }
                ObjectCreateMetadata::ImageView(ImageViewCreateMetadata{ handle, desc, .. }) => {
                    ObjectData::ImageView {
                        handle,
                        source_set: desc.owning_set.clone(),
                    }
                }
                ObjectCreateMetadata::BinarySemaphore(BinarySemaphoreCreateMetadata{ handle, .. }) => {
                    ObjectData::BinarySemaphore { handle }
                }
                ObjectCreateMetadata::TimelineSemaphore(TimelineSemaphoreCreateMetadata{ handle, .. }) => {
                    ObjectData::TimelineSemaphore { handle }
                }
                ObjectCreateMetadata::Event(EventCreateMetadata{ handle, .. }) => {
                    ObjectData::Event { handle }
                }
                ObjectCreateMetadata::Fence(FenceCreateMetadata{ handle, .. }) => {
                    ObjectData::Fence { handle }
                }
            });
        }

        (object_data.into_boxed_slice(), allocations.into_boxed_slice())
    }

    /// Creates objects for a object request description list
    fn create_objects(&self, objects: &[ObjectRequestDescription]) -> (Box<[ObjectData]>, Box<[Allocation]>) {
        let mut objects = self.generate_objects_metadata(objects);
        self.create_objects_for_metadata(objects.as_mut_slice()).map_err(|err| {
            self.destroy_temporary_objects(objects.as_mut_slice()); err
        }).unwrap();

        self.flatten_object_metadata(objects)
    }

    /// Destroys objects previously created using [`ObjectManagerImpl::create_objects`]
    fn destroy_objects(&self, objects: &[ObjectData], allocations: Box<[Allocation]>) {
        for object in objects.iter().rev() {
            match object {
                ObjectData::BufferView { handle, .. } => {
                    unsafe{ self.device.vk().destroy_buffer_view(*handle, None) }
                }
                ObjectData::ImageView { handle, .. } => {
                    unsafe{ self.device.vk().destroy_image_view(*handle, None) }
                }
                ObjectData::Buffer { handle, .. } => {
                    unsafe{ self.device.vk().destroy_buffer(*handle, None) }
                }
                ObjectData::Image { handle, .. } => {
                    unsafe{ self.device.vk().destroy_image(*handle, None) }
                }
                ObjectData::BinarySemaphore { handle, .. } => {
                    unsafe{ self.device.vk().destroy_semaphore(*handle, None) }
                }
                ObjectData::TimelineSemaphore { handle, .. } => {
                    unsafe{ self.device.vk().destroy_semaphore(*handle, None) }
                }
                ObjectData::Event { handle, .. } => {
                    unsafe{ self.device.vk().destroy_event(*handle, None) }
                }
                ObjectData::Fence { handle, .. } => {
                    unsafe{ self.device.vk().destroy_fence(*handle, None) }
                }
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
        SynchronizationGroup::new(self.clone(), self.0.create_group_semaphore(0u64))
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
    fn destroy_group_semaphore(&self, semaphore: vk::Semaphore) {
        self.0.destroy_group_semaphore(semaphore)
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
            vk::BufferUsageFlags::TRANSFER_SRC | vk::BufferUsageFlags::UNIFORM_TEXEL_BUFFER);
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

struct BufferRequestDescription {
    pub description: BufferCreateDesc,
    pub strategy: AllocationStrategy,
}

struct BufferViewRequestDescription {
    pub description: BufferViewCreateDesc,
    /// The set that owns the source buffer of the view. If None the source buffer must be part of
    /// the same set of requests as this request.
    pub owning_set: Option<ObjectSet>,
    pub buffer_id: id::BufferId,
}

struct ImageRequestDescription {
    pub description: ImageCreateDesc,
    pub strategy: AllocationStrategy,
}

struct ImageViewRequestDescription {
    pub description: ImageViewCreateDesc,
    /// The set that owns the source image of the view. If None the source image must be part of
    /// the same set of requests as this request.
    pub owning_set: Option<ObjectSet>,
    pub image_id: id::ImageId,
}

struct BinarySemaphoreRequestDescription {
}

struct TimelineSemaphoreRequestDescription {
    pub initial_value: u64,
}

struct EventRequestDescription {
    pub device_only: bool,
}

struct FenceRequestDescription {
    pub signaled: bool,
}

/// Describes a single object request
enum ObjectRequestDescription {
    Buffer(BufferRequestDescription),
    BufferView(BufferViewRequestDescription),
    Image(ImageRequestDescription),
    ImageView(ImageViewRequestDescription),
    BinarySemaphore(BinarySemaphoreRequestDescription),
    TimelineSemaphore(TimelineSemaphoreRequestDescription),
    Event(EventRequestDescription),
    Fence(FenceRequestDescription),
}

impl ObjectRequestDescription {
    pub fn make_buffer(description: BufferCreateDesc, strategy: AllocationStrategy) -> Self {
        ObjectRequestDescription::Buffer(BufferRequestDescription{
            description,
            strategy
        })
    }

    pub fn make_buffer_view(description: BufferViewCreateDesc, owning_set: Option<ObjectSet>, buffer_id: id::BufferId) -> Self {
        ObjectRequestDescription::BufferView(BufferViewRequestDescription{
            description,
            owning_set,
            buffer_id
        })
    }

    pub fn make_image(description: ImageCreateDesc, strategy: AllocationStrategy) -> Self {
        ObjectRequestDescription::Image(ImageRequestDescription{
            description,
            strategy
        })
    }

    pub fn make_image_view(description: ImageViewCreateDesc, owning_set: Option<ObjectSet>, image_id: id::ImageId) -> Self {
        ObjectRequestDescription::ImageView(ImageViewRequestDescription{
            description,
            owning_set,
            image_id
        })
    }

    pub fn make_binary_semaphore() -> Self {
        Self::BinarySemaphore(BinarySemaphoreRequestDescription{
        })
    }

    pub fn make_timeline_semaphore(initial_value: u64) -> Self {
        Self::TimelineSemaphore(TimelineSemaphoreRequestDescription{
            initial_value
        })
    }

    pub fn make_event(device_only: bool) -> Self {
        Self::Event(EventRequestDescription{
            device_only
        })
    }

    pub fn make_fence(signaled: bool) -> Self {
        Self::Fence(FenceRequestDescription{
            signaled
        })
    }
}
