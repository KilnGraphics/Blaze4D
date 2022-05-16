use std::cmp::Ordering;
use std::fmt::{Debug, Formatter, Pointer};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use ash::vk;
use ash::vk::Handle;

use super::id::ObjectId;
use crate::UUID;

pub trait ObjectSetProvider: Debug {
    fn get_id(&self) -> UUID;

    fn get_handle(&self, id: UUID) -> Option<u64>;
}

#[derive(Clone)]
pub struct ObjectSet(Arc<dyn ObjectSetProvider + Send + Sync>);

impl ObjectSet {
    pub fn new(provider: Arc<dyn ObjectSetProvider + Send + Sync>) -> Self {
        Self(provider)
    }

    pub fn get_provider(&self) -> &Arc<dyn ObjectSetProvider + Send + Sync> {
        &self.0
    }

    pub fn get<ID: ObjectId>(&self, id: ID) -> Option<ID::HandleType> {
        self.0.get_handle(id.as_uuid()).map(|handle| ID::HandleType::from_raw(handle))
    }
}

impl PartialEq for ObjectSet {
    fn eq(&self, other: &Self) -> bool {
        self.0.get_id().eq(&other.0.get_id())
    }
}

impl Eq for ObjectSet {
}

impl PartialOrd for ObjectSet {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.get_id().partial_cmp(&other.0.get_id())
    }
}

impl Ord for ObjectSet {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.get_id().cmp(&other.0.get_id())
    }
}

impl Hash for ObjectSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.get_id().hash(state)
    }
}

impl Debug for ObjectSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        (*self.0).fmt(f)
    }
}