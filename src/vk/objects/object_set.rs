use std::any::Any;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::vk::objects::types::{GenericId, ObjectIdType, ObjectInstanceData, ObjectSetId, UnwrapToInstanceData};

/// A trait that must be implemented by any object set implementation.
pub trait ObjectSetProvider {
    /// Returns the id of this object set.
    fn get_id(&self) -> ObjectSetId;

    fn get_object_data(&self, id: GenericId) -> ObjectInstanceData;

    fn as_any(&self) -> &dyn Any;
}

/// A wrapper type around the [`ObjectSetProvider`] trait.
///
/// Provides a universal object set api.
#[derive(Clone)]
pub struct ObjectSet(Arc<dyn ObjectSetProvider>);

impl ObjectSet {
    /// Creates a new object set from the specified provider.
    pub fn new<T: ObjectSetProvider + 'static>(set: T) -> Self {
        Self(Arc::new(set))
    }

    pub fn get_id(&self) -> ObjectSetId {
        self.0.get_id()
    }

    pub fn get_data<'a, T: super::types::ObjectIdType>(&'a self, id: T) -> &T::InstanceInfo where GenericId: From<T>, ObjectInstanceData<'a>: UnwrapToInstanceData<'a, <T as ObjectIdType>::InstanceInfo> {
        self.0.get_object_data(id.into()).unwrap()
    }

    pub fn as_any(&self) -> &dyn Any {
        self.0.as_any()
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