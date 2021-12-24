//! Management of vulkan objects.
//!
//! Contains structs and enums to manage creation, access to and destruction of vulkan objects.
//!
//! Access to objects is controlled using synchronization groups. All objects belonging to a
//! synchronization group are accessed as one unit protected by a single timeline semaphore. This
//! means 2 objects belonging to the same synchronization group cannot be accessed concurrently but
//! only sequentially.
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

use std::alloc::handle_alloc_error;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::mem::ManuallyDrop;
use std::ptr::hash;
use std::sync::{Arc, LockResult, Mutex, MutexGuard, PoisonError};

use ash::vk;
use ash::vk::{Handle, Image};
use gpu_allocator::{AllocationError, AllocatorDebugSettings, MemoryLocation};
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, Allocator, AllocatorCreateDesc};

use crate::util::id::GlobalId;

use crate::objects::buffer::{BufferCreateDesc, BufferViewCreateDesc};
use crate::objects::image::{ImageCreateDesc, ImageViewCreateDesc};

use super::id;

use synchronization_group::*;
use object_set::*;
use crate::objects::manager::allocator::{BufferRequestDescription, BufferViewRequestDescription, ImageRequestDescription, ImageViewRequestDescription, ObjectRequestDescription};
use crate::util::slice_splitter::Splitter;

#[derive(Debug)]
enum ObjectCreateError<'s> {
    Vulkan(vk::Result),
    Allocation(AllocationError),
    InvalidReference(),
    PoisonedAllocator(PoisonError<MutexGuard<'s, Allocator>>)
}

impl<'s> From<ash::vk::Result> for ObjectCreateError<'s> {
    fn from(err: vk::Result) -> Self {
        ObjectCreateError::Vulkan(err)
    }
}

impl<'s> From<AllocationError> for ObjectCreateError<'s> {
    fn from(err: AllocationError) -> Self {
        ObjectCreateError::Allocation(err)
    }
}

impl<'s> From<PoisonError<MutexGuard<'s, Allocator>>> for ObjectCreateError<'s> {
    fn from(err: PoisonError<MutexGuard<'s, Allocator>>) -> Self {
        ObjectCreateError::PoisonedAllocator(err)
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
    instance: crate::rosella::InstanceContext,
    device: crate::rosella::DeviceContext,

    // We need to ensure the allocator is dropped before the instance and device are
    allocator: ManuallyDrop<Mutex<Allocator>>,
}

impl ObjectManagerImpl {
    fn new(instance: crate::rosella::InstanceContext, device: crate::rosella::DeviceContext) -> Self {
        let allocator: Allocator = Allocator::new(&AllocatorCreateDesc{
            instance: instance.vk().clone(),
            device: device.vk().clone(),
            physical_device: device.get_physical_device().clone(),
            debug_settings: Default::default(),
            buffer_device_address: false
        }).unwrap();

        Self{
            instance,
            device,
            allocator: ManuallyDrop::new(Mutex::new(allocator)),
        }
    }

    fn create_timeline_semaphore(&self, initial_value: u64) -> vk::Semaphore {
        let mut timeline_info = vk::SemaphoreTypeCreateInfo::builder()
            .semaphore_type(vk::SemaphoreType::TIMELINE)
            .initial_value(initial_value);
        let info = vk::SemaphoreCreateInfo::builder().push_next(&mut timeline_info);

        unsafe {
            self.device.vk().create_semaphore(&info.build(), None).unwrap()
        }
    }

    fn destroy_semaphore(&self, semaphore: vk::Semaphore) {
        unsafe {
            self.device.vk().destroy_semaphore(semaphore, None)
        }
    }

    fn destroy_temporary_objects(&self, objects: &mut [TemporaryObjectData], allocator: &mut MutexGuard<Allocator>) {
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
                    allocation.take().map(|alloc| allocator.free(alloc));
                },
                TemporaryObjectData::Image { handle, allocation,  .. } => {
                    if *handle != vk::Image::null() {
                        unsafe { self.device.vk().destroy_image(*handle, None) }
                    }
                    allocation.take().map(|alloc| allocator.free(alloc));
                },
                _ => {}
            }
        }
    }

    fn destroy_temporary_objects_no_lock(&self, objects: &mut [TemporaryObjectData]) {
        let mut guard = self.allocator.lock().unwrap();
        self.destroy_temporary_objects(objects, &mut guard);
    }

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
            let mut allocator = self.allocator.lock()?;
            for object in objects.iter_mut() {
                match object {
                    TemporaryObjectData::Buffer {
                        handle,
                        allocation,
                        desc
                    } => {
                        if allocation.is_none() {
                            let requirements = unsafe {
                                self.device.vk().get_buffer_memory_requirements(*handle)
                            };

                            let alloc_desc = AllocationCreateDesc{
                                name: "",
                                requirements,
                                location: desc.memory_location,
                                linear: true
                            };

                            *allocation = Some(allocator.allocate(&alloc_desc)?);
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
                            let requirements = unsafe {
                                self.device.vk().get_image_memory_requirements(*handle)
                            };

                            let alloc_desc = AllocationCreateDesc{
                                name: "",
                                requirements,
                                location: desc.memory_location,
                                linear: false
                            };

                            *allocation = Some(allocator.allocate(&alloc_desc)?);
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
            let (mut split, elem) = Splitter::new(objects, i);

            match elem {
                TemporaryObjectData::BufferView {
                    handle,
                    desc
                } => {
                    if handle != &vk::BufferView::null() {
                        let buffer = match desc.owning_set.as_ref() {
                            Some(set) => {
                                set.get_buffer_handle(desc.buffer_id).ok_or(ObjectCreateError::InvalidReference())?
                            }
                            None => {
                                let index = desc.buffer_id.get_index() as usize;
                                match split.get(index).ok_or(ObjectCreateError::InvalidReference())? {
                                    TemporaryObjectData::Buffer { handle, .. } => *handle,
                                    _ => return Err(ObjectCreateError::InvalidReference())
                                }
                            }
                        };

                        let create_info = vk::BufferViewCreateInfo::builder()
                            .buffer(buffer)
                            .format(desc.description.format)
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
                                set.get_image_handle(desc.image_id).ok_or(ObjectCreateError::InvalidReference())?
                            }
                            None => {
                                let index = desc.image_id.get_index() as usize;
                                match split.get(index).ok_or(ObjectCreateError::InvalidReference())? {
                                    TemporaryObjectData::Image { handle, .. } => *handle,
                                    _ => return Err(ObjectCreateError::InvalidReference())
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

    fn flatten_temporary_object_data(&self, objects: Vec<TemporaryObjectData>) -> (Box<[ObjectData]>, AllocationMeta) {
        let mut allocations = Vec::new();
        let mut object_data = Vec::with_capacity(objects.len());

        for mut object in objects.into_iter() {
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

        (object_data.into_boxed_slice(), AllocationMeta{ allocations: allocations.into_boxed_slice() })
    }

    fn create_objects(&self, objects: &[ObjectRequestDescription]) -> (Box<[ObjectData]>, AllocationMeta) {
        let mut objects = self.create_temporary_object_data(objects);
        self.create_temporary_objects(objects.as_mut_slice()).map_err(|err| {
            let mut guard = self.allocator.lock().unwrap();
            self.destroy_temporary_objects(objects.as_mut_slice(), &mut guard); err
        }).unwrap();

        self.flatten_temporary_object_data(objects)
    }

    fn destroy_objects(&self, objects: &[ObjectData], allocation: &AllocationMeta) {
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

        let mut guard = self.allocator.lock().unwrap();
        for allocation in allocation.allocations.as_ref() {
            guard.free(allocation.clone());
        }
    }
}

impl Drop for ObjectManagerImpl {
    fn drop(&mut self) {
        unsafe{ ManuallyDrop::drop(&mut self.allocator) }
    }
}

/// Public object manager api.
///
/// This is a smart pointer reference to an internal struct.
pub struct ObjectManager(Arc<ObjectManagerImpl>);

impl ObjectManager {
    /// Creates a new ObjectManager
    pub fn new(instance: crate::rosella::InstanceContext, device: crate::rosella::DeviceContext) -> Self {
        Self(Arc::new(ObjectManagerImpl::new(instance, device)))
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

    // Internal function that destroys a semaphore created for a synchronization group
    fn destroy_semaphore(&self, semaphore: vk::Semaphore) {
        self.0.destroy_semaphore(semaphore)
    }

    fn create_objects(&self, objects: &[ObjectRequestDescription]) -> (Box<[ObjectData]>, AllocationMeta) {
        self.0.create_objects(objects)
    }

    // Internal function that destroys objects and allocations created for a object set
    fn destroy_objects(&self, objects: &[ObjectData], allocation: &AllocationMeta) {
        self.0.destroy_objects(objects, allocation)
    }
}

impl Clone for ObjectManager {
    fn clone(&self) -> Self {
        Self( self.0.clone() )
    }
}

/// Internal struct containing information about a memory allocation.
///
/// These structs dont have to have a 1 to 1 mapping to objects. A allocation can back multiple
/// objects or a object can be backed by multiple allocations.
struct AllocationMeta {
    allocations: Box<[Allocation]>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create() -> ObjectManager {
        let (instance, device) = crate::test::make_headless_instance_device();
        ObjectManager::new(instance, device)
    }

    #[test]
    fn create_destroy() {
        let (instance, device) = crate::test::make_headless_instance_device();
        let manager = ObjectManager::new(instance, device);
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

        assert_eq!(set.get_synchronization_group(), &group);

        assert!(set.get_buffer_handle(id).is_some());

        drop(set);
    }
}