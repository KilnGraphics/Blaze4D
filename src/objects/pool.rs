use std::any::Any;
use std::sync::{Arc, LockResult};

use ash::vk;

use crate::objects::id;

pub trait ObjectPool {

}

pub struct PoolAccess {
    semaphore: vk::Semaphore,
    base_offset: u64,
}

pub trait PoolGuard {
    fn enqueue_access(&mut self) -> PoolAccess;
}

pub trait LeaseManager {
    fn get_pool(&self) -> Arc<dyn ObjectPool>;

    fn lock_pool(&mut self) -> Option<Box<dyn PoolGuard>>;

    fn release_lease(&mut self, meta: Option<Box<dyn Any>>, objects: &[id::GenericId]);
}

pub struct ObjectLease {
    manager: Box<dyn LeaseManager>,
    meta: Option<Box<dyn Any>>,
    objects: Box<[id::GenericId]>,
}

impl Drop for ObjectLease {
    fn drop(&mut self) {
        self.manager.release_lease(self.meta.take(), self.objects.as_ref());
    }
}