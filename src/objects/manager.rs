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

use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, LockResult, Mutex, MutexGuard};

use ash::vk;
use ash::vk::Handle;
use crate::util::id::GlobalId;

use super::id;

/// Contains all the information (type, flags, allocation requirements etc.) about how an object
/// should be created.
#[non_exhaustive]
pub enum ObjectCreateMeta {
    Buffer(super::buffer::BufferCreateMeta, AllocationCreateMeta),
    BufferView(super::buffer::BufferViewCreateMeta, usize),
    Image(super::image::ImageCreateMeta, AllocationCreateMeta),
    ImageView(super::image::ImageViewCreateMeta, usize),
    Event(),
}

/// Contains information about how memory should be allocated for an object.
pub struct AllocationCreateMeta {
}

/// Wrapper type that is passed to the object set create function. The create function will store
/// the assigned id of the object in this type which can then later be retrieved by the calling
/// code.
pub struct ObjectCreateRequest {
    meta: ObjectCreateMeta,
    id: Option<id::GenericId>,
}

impl ObjectCreateRequest {
    /// Creates a new request without a resolved id.
    pub fn new(meta: ObjectCreateMeta) -> Self {
        Self{ meta, id: None }
    }

    /// Returns the create metadata.
    pub fn get_meta(&self) -> &ObjectCreateMeta {
        &self.meta
    }

    /// Updates the stored id.
    pub fn resolve(&mut self, id: id::GenericId) {
        self.id = Some(id)
    }

    /// Retrieves the currently stored id.
    pub fn get_id(&self) -> Option<id::GenericId> {
        self.id
    }

    /// Retrieves the currently stored id as an id of specified type.
    pub fn get_id_typed<const TYPE: u8>(&self) -> Option<id::ObjectId<TYPE>> {
        self.id.map(|id| id.downcast::<TYPE>()).flatten()
    }
}

// Internal implementation of the object manager
struct ObjectManagerImpl {
    instance: Arc<crate::rosella::InstanceContext>,
    device: Arc<crate::rosella::DeviceContext>,
}

impl ObjectManagerImpl {
    fn new(instance: Arc<crate::rosella::InstanceContext>, device: Arc<crate::rosella::DeviceContext>) -> Self {
        Self{ instance, device }
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

    fn destroy_objects(&self, objects: &[ObjectData], allocations: &[AllocationMeta]) {
        todo!()
    }
}

/// Public object manager api.
///
/// This is a smart pointer reference to an internal struct.
pub struct ObjectManager(Arc<ObjectManagerImpl>);

impl ObjectManager {
    /// Creates a new ObjectManager
    pub fn new(instance: Arc<crate::rosella::InstanceContext>, device: Arc<crate::rosella::DeviceContext>) -> Self {
        Self(Arc::new(ObjectManagerImpl::new(instance, device)))
    }

    /// Creates a new synchronization group managed by this object manager
    pub fn create_synchronization_group(&self) -> SynchronizationGroup {
        SynchronizationGroup::new(self.clone(), self.0.create_timeline_semaphore(0u64))
    }

    /// Creates a new object set managed by this object manager
    ///
    /// The synchronization group *can* be from a different object manager however no validation is
    /// performed with respect to any vulkan requirements. (For example ensuring that both control
    /// the same device).
    pub fn create_object_set(&self, objects: &mut [ObjectCreateRequest], synchronization_group: SynchronizationGroup) -> ObjectSet {
        todo!()
    }

    // Internal function that destroys a semaphore created for a synchronization group
    fn destroy_semaphore(&self, semaphore: vk::Semaphore) {
        self.0.destroy_semaphore(semaphore)
    }

    // Internal function that destroys objects and allocations created for a object set
    fn destroy_objects(&self, objects: &[ObjectData], allocations: &[AllocationMeta]) {
        self.0.destroy_objects(objects, allocations)
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

// Internal struct containing the semaphore payload and metadata
struct SyncData {
    semaphore: vk::Semaphore,
    last_access: u64,
}

impl SyncData {
    fn enqueue_access(&mut self, step_count: u64) -> AccessInfo {
        let begin_access = self.last_access;
        let end_access = begin_access + step_count;
        self.last_access = end_access;

        AccessInfo{
            semaphore: self.semaphore,
            begin_access,
            end_access,
        }
    }
}

// Internal implementation of the synchronization group
struct SynchronizationGroupImpl {
    group_id: GlobalId,
    sync_data: Mutex<SyncData>,
    manager: ObjectManager,
}

impl SynchronizationGroupImpl {
    fn new(manager: ObjectManager, semaphore: vk::Semaphore) -> Self {
        Self{ group_id: GlobalId::new(), sync_data: Mutex::new(SyncData{ semaphore, last_access: 0u64 }), manager }
    }

    fn get_group_id(&self) -> GlobalId {
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

impl Eq for SynchronizationGroupImpl {
}

impl PartialOrd for SynchronizationGroupImpl {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.group_id.partial_cmp(&other.group_id)
    }
}

impl Ord for SynchronizationGroupImpl {
    fn cmp(&self, other: &Self) -> Ordering {
        self.group_id.cmp(&other.group_id)
    }
}

/// Public synchronization group api.
///
/// This is a smart pointer reference to an internal struct.
pub struct SynchronizationGroup(Arc<SynchronizationGroupImpl>);

impl SynchronizationGroup {
    fn new(manager: ObjectManager, semaphore: vk::Semaphore) -> Self {
        Self(Arc::new(SynchronizationGroupImpl::new(manager, semaphore)))
    }

    /// Returns the object manager managing this synchronization group
    pub fn get_manager(&self) -> &ObjectManager {
        &self.0.manager
    }

    /// Enqueues an access to the resources protected by this group.
    ///
    /// `step_count` is the number of steps added to the semaphore payload.
    ///
    /// If access to multiple groups is needed simultaneously; accesses **must not** be queued
    /// individually but by using a synchronization group set. Not doing so may result in a
    /// deadlock when waiting for the semaphores.
    pub fn enqueue_access(&self, step_count: u64) -> AccessInfo {
        self.0.lock().unwrap().enqueue_access(step_count)
    }
}

impl Clone for SynchronizationGroup {
    fn clone(&self) -> Self {
        Self( self.0.clone() )
    }
}

impl PartialEq for SynchronizationGroup {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for SynchronizationGroup {
}

impl PartialOrd for SynchronizationGroup {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for SynchronizationGroup {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl Hash for SynchronizationGroup {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.group_id.hash(state)
    }
}

/// Stores information for a single accesses queued up in a synchronization group.
pub struct AccessInfo {
    /// The timeline semaphore protecting the group.
    pub semaphore: vk::Semaphore,

    /// The value of the semaphore when the access may begin execution.
    pub begin_access: u64,

    /// The value the semaphore must have to signal the end of the access.
    pub end_access: u64,
}

#[non_exhaustive]
enum ObjectData {
    Buffer{
        handle: vk::Buffer,
        meta: super::buffer::BufferMeta,
    },
    Image {
        handle: vk::Image,
    }
}

impl ObjectData {
    fn get_raw_handle(&self) -> u64 {
        match self {
            ObjectData::Buffer { handle, .. } => handle.as_raw(),
            ObjectData::Image { handle, .. } => handle.as_raw(),
        }
    }
}

// Internal implementation of the object set
struct ObjectSetImpl {
    group: SynchronizationGroup,
    set_id: GlobalId,
    objects: Box<[ObjectData]>,
    allocations: Box<[AllocationMeta]>,
}

impl ObjectSetImpl {
    fn new(synchronization_group: SynchronizationGroup, objects: Box<[ObjectData]>, allocations: Box<[AllocationMeta]>) -> Self {
        Self{
            group: synchronization_group,
            set_id: GlobalId::new(),
            objects,
            allocations
        }
    }

    fn get_raw_handle(&self, id: id::GenericId) -> Option<u64> {
        if id.get_global_id() != self.set_id {
            return None;
        }

        // Invalid local id but matching global is a serious error
        Some(self.objects.get(id.get_index() as usize).unwrap().get_raw_handle())
    }

    fn get_buffer_handle(&self, id: id::BufferId) -> Option<vk::Buffer> {
        if id.get_global_id() != self.set_id {
            return None;
        }

        // Invalid local id but matching global is a serious error
        match self.objects.get(id.get_index() as usize).unwrap() {
            ObjectData::Buffer { handle, .. } => Some(*handle),
            _ => panic!("Object type mismatch"),
        }
    }

    fn get_image_handle(&self, id: id::ImageId) -> Option<vk::Image> {
        if id.get_global_id() != self.set_id {
            return None;
        }

        // Invalid local id but matching global is a serious error
        match self.objects.get(id.get_index() as usize).unwrap() {
            ObjectData::Image { handle, .. } => Some(*handle),
            _ => panic!("Object type mismatch"),
        }
    }
}

// Needed because the SynchronizationSet mutex also protects the ObjectSet
unsafe impl Sync for ObjectSetImpl {
}

impl Drop for ObjectSetImpl {
    fn drop(&mut self) {
        self.group.get_manager().destroy_objects(self.objects.as_ref(), self.allocations.as_ref())
    }
}

impl PartialEq for ObjectSetImpl {
    fn eq(&self, other: &Self) -> bool {
        self.set_id.eq(&other.set_id)
    }
}

impl Eq for ObjectSetImpl {
}

impl PartialOrd for ObjectSetImpl {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.set_id.partial_cmp(&other.set_id)
    }
}

impl Ord for ObjectSetImpl {
    fn cmp(&self, other: &Self) -> Ordering {
        self.set_id.cmp(&other.set_id)
    }
}


/// Public object set api.
///
/// This is a smart pointer reference to an internal struct.
pub struct ObjectSet(Arc<ObjectSetImpl>);

impl ObjectSet {
    fn new(synchronization_group: SynchronizationGroup, objects: Box<[ObjectData]>, allocations: Box<[AllocationMeta]>) -> Self {
        Self(Arc::new(ObjectSetImpl::new(synchronization_group, objects, allocations)))
    }

    /// Returns the synchronization group that controls access to this object set.
    pub fn get_synchronization_group(&self) -> &SynchronizationGroup {
        &self.0.group
    }

    /// Returns the handle of an object that is part of this object set.
    ///
    /// If the id is not part of the object set (i.e. the global id does not match) None will be
    /// returned. If the id is invalid (matching global id but local id is invalid) the function
    /// panics.
    pub fn get_raw_handle(&self, id: id::GenericId) -> Option<u64> {
        self.0.get_raw_handle(id)
    }

    /// Returns the handle of a buffer that is part of this object set.
    ///
    /// If the id is not part of the object set (i.e. the global id does not match) None will be
    /// returned. If the id is invalid (matching global id but local id is invalid or object type
    /// is not a buffer) the function panics.
    pub fn get_buffer_handle(&self, id: id::BufferId) -> Option<vk::Buffer> {
        self.0.get_buffer_handle(id)
    }

    /// Returns the handle of a image that is part of this object set.
    ///
    /// If the id is not part of the object set (i.e. the global id does not match) None will be
    /// returned. If the id is invalid (matching global id but local id is invalid or object type
    /// is not a image) the function panics.
    pub fn get_image_handle(&self, id: id::ImageId) -> Option<vk::Image> {
        self.0.get_image_handle(id)
    }
}

impl Clone for ObjectSet {
    fn clone(&self) -> Self {
        Self( self.0.clone() )
    }
}

impl PartialEq for ObjectSet {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for ObjectSet {
}

impl PartialOrd for ObjectSet {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for ObjectSet {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl Hash for ObjectSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.set_id.hash(state)
    }
}

pub struct SynchronizationGroupSet {
    groups: Box<[SynchronizationGroup]>,
}

impl SynchronizationGroupSet {
    pub fn new(groups: &std::collections::BTreeSet<SynchronizationGroup>) -> Self {
        // BTreeSet is required to guarantee the groups are sorted

        let collected : Vec<_> = groups.into_iter().map(|group| group.clone()).collect();
        Self{ groups: collected.into_boxed_slice() }
    }

    pub fn enqueue_access(&self, step_counts: &[u64]) -> Box<[AccessInfo]> {
        if self.groups.len() != step_counts.len() {
            panic!("Step counts length mismatch")
        }

        let mut guards = Vec::with_capacity(self.groups.len());

        for group in self.groups.iter() {
            guards.push(group.0.lock().unwrap())
        }

        let mut accesses = Vec::with_capacity(self.groups.len());

        for (i, mut guard) in guards.into_iter().enumerate() {
            accesses.push(guard.enqueue_access(*step_counts.get(i).unwrap()));
        }

        accesses.into_boxed_slice()
    }
}