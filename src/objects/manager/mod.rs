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
mod resource_object_set;

use std::sync::Arc;

use ash::vk;

use synchronization_group::*;
use crate::objects::manager::allocator::*;
use crate::util::slice_splitter::Splitter;

pub use object_set::ObjectSetProvider;
use crate::objects::manager::resource_object_set::{ObjectCreateError, ResourceObjectCreateMetadata, ResourceObjectCreator, ResourceObjectData, ResourceObjectSetBuilder};

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

    fn create_resource_objects(&self, objects: &mut Box<[ResourceObjectCreateMetadata]>) -> Result<(), ObjectCreateError> {
        for i in 0..objects.len() {
            let (splitter, current) = Splitter::new(objects.as_mut(), i);
            current.create(&self.device, &self.allocator, &splitter)?
        }
        Ok(())
    }

    fn abort_resource_objects(&self, objects: &mut Box<[ResourceObjectCreateMetadata]>) {
        for object in objects.iter_mut().rev() {
            object.abort(&self.device, &self.allocator)
        }
    }

    fn reduce_resource_objects(&self, objects: Box<[ResourceObjectCreateMetadata]>) -> (Box<[ResourceObjectData]>, Box<[Allocation]>){
        let mut data = Vec::with_capacity(objects.len());
        let mut allocations = Vec::new();

        for object in objects.into_vec() {
            let (d, alloc) = object.reduce();
            data.push(d);

            match alloc {
                Some(alloc) => allocations.push(alloc),
                None => {}
            }
        }

        (data.into_boxed_slice(), allocations.into_boxed_slice())
    }

    fn destroy_resource_objects(&self, objects: Box<[ResourceObjectData]>, allocations: Box<[Allocation]>) {
        for object in objects.into_vec().into_iter().rev() {
            object.destroy(&self.device)
        }
        for allocation in allocations.into_vec() {
            self.allocator.free(allocation)
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
    pub fn create_resource_object_set(&self, synchronization_group: SynchronizationGroup) -> ResourceObjectSetBuilder {
        // TODO test manager equality

        ResourceObjectSetBuilder::new(synchronization_group)
    }

    // Internal function that destroys a semaphore created for a synchronization group
    fn destroy_group_semaphore(&self, semaphore: vk::Semaphore) {
        self.0.destroy_group_semaphore(semaphore)
    }

    fn build_resource_objects(&self, mut objects: Box<[ResourceObjectCreateMetadata]>) -> (Box<[ResourceObjectData]>, Box<[Allocation]>) {
        let result = self.0.create_resource_objects(&mut objects);
        if result.is_err() {
            self.0.abort_resource_objects(&mut objects);
            panic!("Error during object creation")
        }

        self.0.reduce_resource_objects(objects)
    }

    fn destroy_resource_objects(&self, objects: Box<[ResourceObjectData]>, allocations: Box<[Allocation]>) {
        self.0.destroy_resource_objects(objects, allocations)
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
}