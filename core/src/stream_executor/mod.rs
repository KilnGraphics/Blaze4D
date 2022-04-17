mod commands;

use std::sync::Arc;
use crate::objects::id::{SemaphoreId, SwapchainId};
use crate::objects::ObjectSet;

struct StreamExecutorImpl {

}

pub struct StreamExecutor(Arc<StreamExecutorImpl>);

impl StreamExecutor {
    pub fn acquire_objects(&self, objects: ObjectSet, wait_semaphores: Box<[SemaphoreId]>) -> ObjectsToken {
        todo!()
    }
}

/// Represents a object collection acquired by a StreamExecutor
pub struct ObjectsToken {

}

impl ObjectsToken {
    pub fn release_objects(self) {

    }
}