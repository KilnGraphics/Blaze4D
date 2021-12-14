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

use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, LockResult, Mutex, MutexGuard};

use ash::vk;
use ash::vk::Handle;
use gpu_allocator::{AllocationError, AllocatorDebugSettings, MemoryLocation};
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, Allocator, AllocatorCreateDesc};

use crate::util::id::GlobalId;

use crate::objects::buffer::{BufferCreateInfo, BufferViewCreateInfo};
use crate::objects::image::{ImageCreateMeta, ImageViewCreateMeta};

use super::id;

use synchronization_group::*;
use object_set::*;

enum ObjectCreateError {
    Vulkan(vk::Result),
    Allocation(AllocationError)
}

impl From<ash::vk::Result> for ObjectCreateError {
    fn from(err: vk::Result) -> Self {
        ObjectCreateError::Vulkan(err)
    }
}

impl From<AllocationError> for ObjectCreateError {
    fn from(err: AllocationError) -> Self {
        ObjectCreateError::Allocation(err)
    }
}

// Internal implementation of the object manager
struct ObjectManagerImpl {
    instance: crate::rosella::InstanceContext,
    device: crate::rosella::DeviceContext,
    allocator: Allocator,
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
            allocator,
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

    fn create_buffer(&mut self, info: &BufferCreateInfo, location: MemoryLocation, allocations: &mut Vec<Allocation>) -> Result<ObjectData, ObjectCreateError> {
        // Make sure that any potential panic in push gets triggered here so that we dont have dangling objects
        allocations.reserve(allocations.len() + 1);

        let create_info = vk::BufferCreateInfo::builder()
            .size(info.size)
            .usage(info.usage_flags)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe {
             self.device.vk().create_buffer(&create_info.build(), None)?
        };

        let requirements = unsafe {
            self.device.vk().get_buffer_memory_requirements(buffer)
        };

        let allocation_info = AllocationCreateDesc{
            name: "",
            requirements,
            location,
            linear: true
        };

        let allocation = self.allocator.allocate(&allocation_info).map_err(|err| {
            unsafe{ self.device.vk().destroy_buffer(buffer, None); };
            err
        })?;

        unsafe { self.device.vk().bind_buffer_memory(buffer, allocation.memory(), allocation.offset()) }.map_err(|err| {
            self.allocator.free(allocation.clone());
            unsafe { self.device.vk().destroy_buffer(buffer, None); };
            err
        })?;

        allocations.push(allocation);
        Ok(ObjectData::Buffer{ handle: buffer })
    }

    fn create_object(&self, info: &ObjectCreateInfo, objects: &mut Vec<ObjectData>, allocations: &mut Vec<Allocation>) {
        match info {
            ObjectCreateInfo::Buffer(_, _) => {}
            ObjectCreateInfo::InternalBufferView(_, _) => {}
            ObjectCreateInfo::ExternalBufferView(_, _, _) => {}
            ObjectCreateInfo::Image(_, _) => {}
            ObjectCreateInfo::ImageView(_, _) => {}
            ObjectCreateInfo::Event() => {}
        }
    }

    fn create_objects(&self, objects: &[ObjectCreateInfo]) -> (Box<[ObjectData]>, AllocationMeta) {
        todo!()
    }

    fn destroy_objects(&self, objects: &[ObjectData], allocation: &AllocationMeta) {
        todo!()
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

    fn create_objects(&self, objects: &[ObjectCreateInfo]) -> (Box<[ObjectData]>, AllocationMeta) {
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
}


