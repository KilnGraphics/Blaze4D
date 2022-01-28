use std::any::Any;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use crate::objects::manager::synchronization_group::SynchronizationGroup;

use ash::vk::Handle;
use crate::objects::id;
use crate::objects::id::{ObjectIdType, ObjectSetId};

/// A trait that must be implemented by any object set implementation.
pub trait ObjectSetProvider {
    fn get_id(&self) -> ObjectSetId;

    /// Returns the handle for an object id.
    ///
    /// #Panics
    /// If the id does not map to an object in the set this function must panic.
    fn get_raw_handle(&self, id: id::GenericId) -> u64;

    /// Returns the synchronization group associated with a object.
    ///
    /// An object may not have a associated synchronization group in which case None should be
    /// returned. Similarly for objects which do not need a synchronization group this function may
    /// still return a synchronization group. (This is to allow object sets to return the same group
    /// for all objects).
    ///
    /// #Panics
    /// If the id does not map to an object in the set this function must panic.
    fn get_synchronization_group(&self, id: id::GenericId) -> Option<SynchronizationGroup>;

    fn as_any(&self) -> &dyn Any;
}

/// A wrapper type around the [`ObjectSetProvider`] trait.
///
/// Provides a uniform object set api.
#[derive(Clone)]
pub struct ObjectSet(Arc<dyn ObjectSetProvider>);

impl ObjectSet {
    /// Creates a new object set from the specified provider.
    pub fn new<T: ObjectSetProvider + 'static>(set: T) -> Self {
        Self(Arc::new(set))
    }

    /// Returns the UUID of this object set
    pub fn get_id(&self) -> ObjectSetId {
        self.0.get_id()
    }

    /// Returns a handle for some object stored in this set.
    ///
    /// #Panics
    /// If the id does not map to an object in this set the function will panic.
    pub fn get_handle<TYPE: ObjectIdType>(&self, id: TYPE) -> TYPE::Handle {
        TYPE::Handle::from_raw(self.0.get_raw_handle(id.as_generic()))
    }

    /// Returns the synchronization group associated with a object stored in this set.
    ///
    /// An object may not have a associated synchronization group in which case None should be
    /// returned. Similarly for objects which do not need a synchronization group this function may
    /// still return a synchronization group. (This is to allow object sets to return the same group
    /// for all objects).
    ///
    /// #Panics
    /// If the id does not map to an object in this set this function will panic.
    pub fn get_synchronization_group<TYPE: ObjectIdType>(&self, id: TYPE) -> Option<SynchronizationGroup> {
        self.0.get_synchronization_group(id.as_generic())
    }

    /// Returns a any reference to the wrapped [`ObjectSetProvider`]
    pub fn get_any(&self) -> &dyn Any {
        self.0.as_ref().as_any()
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