use std::any::Any;
use std::cmp::Ordering;
use std::sync::{Arc, LockResult, Mutex, MutexGuard, PoisonError};

use ash::vk;

use super::memory;
use super::id;

#[non_exhaustive]
pub enum ObjectCreateMeta {
    Buffer(super::buffer::BufferCreateMeta, memory::AllocationCreateMeta)
}

pub struct ObjectCreateRequest {
    meta: ObjectCreateMeta,
    id: Option<id::GenericId>,
}

impl ObjectCreateRequest {
    pub fn new(meta: ObjectCreateMeta) -> Self {
        Self{ meta, id: None }
    }

    pub fn resolve(&mut self, id: id::GenericId) {
        self.id = Some(id)
    }

    pub fn get_id(&self) -> Option<id::GenericId> {
        self.id
    }
}

struct ObjectManagerImpl {
    instance: Arc<crate::rosella::InstanceContext>,
    device: Arc<crate::rosella::DeviceContext>,
}

impl ObjectManagerImpl {
    fn new(instance: Arc<crate::rosella::InstanceContext>, device: Arc<crate::rosella::DeviceContext>) -> Self {
        Self{ instance, device }
    }

    fn create_timeline_semaphore(&self) -> vk::Semaphore {
        let mut timeline_info = vk::SemaphoreTypeCreateInfo::builder()
            .semaphore_type(vk::SemaphoreType::TIMELINE)
            .initial_value(0);
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
}

struct ObjectManager(Arc<ObjectManagerImpl>);

impl ObjectManager {
    pub fn new(instance: Arc<crate::rosella::InstanceContext>, device: Arc<crate::rosella::DeviceContext>) -> Self {
        Self(Arc::new(ObjectManagerImpl::new(instance, device)))
    }

    pub fn create_synchronization_group(&self) -> SynchronizationGroup {
        SynchronizationGroup::new(self.clone(), self.0.create_timeline_semaphore())
    }

    pub fn create_object_set(&self, objects: &mut [ObjectCreateRequest]) -> ObjectSet {
        todo!()
    }

    fn destroy_semaphore(&self, semaphore: vk::Semaphore) {
        self.0.destroy_semaphore(semaphore)
    }
}

impl Clone for ObjectManager {
    fn clone(&self) -> Self {
        Self( self.0.clone() )
    }
}


struct SyncData {
    semaphore: vk::Semaphore,
    last_access: u64,
}

struct SynchronizationGroupImpl {
    group_id: u64,
    sync_data: Mutex<SyncData>,
    manager: ObjectManager,
}

impl SynchronizationGroupImpl {
    fn new(manager: ObjectManager, semaphore: vk::Semaphore) -> Self {
        Self{ group_id: id::make_global_id(), sync_data: Mutex::new(SyncData{ semaphore, last_access: 0u64 }), manager }
    }

    fn get_group_id(&self) -> u64 {
        self.group_id
    }

    fn lock(&self) -> LockResult<MutexGuard<SyncData>> {
        self.sync_data.lock()
    }
}

impl Drop for SynchronizationGroupImpl {
    fn drop(&mut self) {
        self.manager.destroy_semaphore(self.sync_data.get_mut().unwrap().semaphore)
    }
}

impl PartialEq for SynchronizationGroupImpl {
    fn eq(&self, other: &Self) -> bool {
        self.group_id == other.group_id
    }
}

impl PartialOrd for SynchronizationGroupImpl {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.group_id.partial_cmp(&other.group_id)
    }
}

pub struct SynchronizationGroup(Arc<SynchronizationGroupImpl>);

impl SynchronizationGroup {
    fn new(manager: ObjectManager, semaphore: vk::Semaphore) -> Self {
        Self(Arc::new(SynchronizationGroupImpl::new(manager, semaphore)))
    }
}

impl PartialEq for SynchronizationGroup {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl PartialOrd for SynchronizationGroup {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

struct ObjectSetImpl {
    group: SynchronizationGroup,
    objects: Box<[()]>,
}

impl ObjectSetImpl {

}

unsafe impl Sync for ObjectSetImpl {
}

pub struct ObjectSet(Arc<ObjectSetImpl>);

impl ObjectSet {
}