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

mod allocator;

use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, LockResult, Mutex, MutexGuard};

use ash::vk;
use ash::vk::Handle;
use gpu_allocator::AllocatorDebugSettings;
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};

use crate::util::id::GlobalId;

use allocator::*;
use crate::objects::buffer::{BufferCreateInfo, BufferViewCreateInfo};
use crate::objects::image::{ImageCreateMeta, ImageViewCreateMeta};

use super::id;

// Internal implementation of the object manager
struct ObjectManagerImpl {
    instance: Arc<crate::rosella::InstanceContext>,
    device: Arc<crate::rosella::DeviceContext>,
}

impl ObjectManagerImpl {
    fn new(instance: Arc<crate::rosella::InstanceContext>, device: Arc<crate::rosella::DeviceContext>) -> Self {
        Self{
            instance,
            device,
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

    fn create_object(&self, info: &ObjectCreateInfo, objects: Vec<ObjectData>, allocations: Vec<gpu_allocator::vulkan::Allocation>) {

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
    pub fn new(instance: Arc<crate::rosella::InstanceContext>, device: Arc<crate::rosella::DeviceContext>) -> Self {
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
    },
    BufferView{
        handle: vk::BufferView,
    },
    Image {
        handle: vk::Image,
    }
}

impl ObjectData {
    fn get_raw_handle(&self) -> u64 {
        match self {
            ObjectData::Buffer { handle, .. } => handle.as_raw(),
            ObjectData::BufferView {handle, .. } => handle.as_raw(),
            ObjectData::Image { handle, .. } => handle.as_raw(),
        }
    }
}

/// Contains all the information (type, flags, allocation requirements etc.) about how an object
/// should be created.
enum ObjectCreateInfo {
    Buffer(BufferCreateInfo, gpu_allocator::MemoryLocation),
    InternalBufferView(super::buffer::BufferViewCreateInfo, usize),
    ExternalBufferView(super::buffer::BufferViewCreateInfo, ObjectSet, id::BufferId),
    Image(super::image::ImageCreateMeta, gpu_allocator::MemoryLocation),
    ImageView(super::image::ImageViewCreateMeta, usize),
    Event(),
}

pub struct ObjectSetBuilder {
    synchronization_group: SynchronizationGroup,
    set_id: GlobalId,
    requests: Vec<ObjectCreateInfo>,
}

impl ObjectSetBuilder {
    fn new(synchronization_group: SynchronizationGroup) -> Self {
        Self {
            synchronization_group,
            set_id: GlobalId::new(),
            requests: Vec::new(),
        }
    }

    fn with_capacity(synchronization_group: SynchronizationGroup, capacity: usize) -> Self {
        Self {
            synchronization_group,
            set_id: GlobalId::new(),
            requests: Vec::with_capacity(capacity),
        }
    }

    pub fn add_default_gpu_only_buffer(&mut self, info: BufferCreateInfo) -> id::BufferId {
        let index = self.requests.len();

        self.requests.push(ObjectCreateInfo::Buffer(
            info,
            gpu_allocator::MemoryLocation::GpuOnly
        ));

        id::BufferId::new(self.set_id, index as u64)
    }

    pub fn add_default_gpu_cpu_buffer(&mut self, info: BufferCreateInfo) -> id::BufferId {
        let index = self.requests.len();

        self.requests.push(ObjectCreateInfo::Buffer(
            info,
            gpu_allocator::MemoryLocation::CpuToGpu
        ));

        id::BufferId::new(self.set_id, index as u64)
    }

    pub fn add_internal_buffer_view(&mut self, info: BufferViewCreateInfo, buffer: id::BufferId) -> id::BufferViewId {
        if buffer.get_global_id() != self.set_id {
            panic!("Buffer global id does not match set id")
        }

        let index = self.requests.len();

        self.requests.push(ObjectCreateInfo::InternalBufferView(
            info,
            buffer.get_index() as usize
        ));

        id::BufferViewId::new(self.set_id, index as u64)
    }

    pub fn add_external_buffer_view(&mut self, info: BufferViewCreateInfo, set: ObjectSet, buffer: id::BufferId) -> id::BufferViewId {
        if buffer.get_global_id() != set.get_set_id() {
            panic!("Buffer global id does not match set id")
        }

        if *set.get_synchronization_group() != self.synchronization_group {
            panic!("Buffer does not match internal synchronization group")
        }

        let index = self.requests.len();

        self.requests.push(ObjectCreateInfo::ExternalBufferView(
            info,
            set,
            buffer
        ));

        id::BufferViewId::new(self.set_id, index as u64)
    }

    pub fn add_default_gpu_only_image(&mut self, info: ImageCreateMeta) -> id::ImageId {
        todo!()
    }

    pub fn add_default_gpu_cpu_image(&mut self, info: ImageCreateMeta) -> id::ImageId {
        todo!()
    }

    pub fn add_internal_image_view(&mut self, info: ImageViewCreateMeta, image: id::ImageId) -> id::ImageViewId {
        todo!()
    }

    pub fn add_external_image_view(&mut self, info: ImageViewCreateMeta, set: ObjectSet, image: id::ImageId) -> id::ImageViewId {
        todo!()
    }

    pub fn build(self) -> ObjectSet {
        let (objects, allocation) = self.synchronization_group.get_manager().create_objects(self.requests.as_slice());
        ObjectSet::new(self.synchronization_group, objects, allocation)
    }
}

/// Wrapper type that is passed to the object set create function. The create function will store
/// the assigned id of the object in this type which can then later be retrieved by the calling
/// code.
pub struct ObjectCreateRequest {
    meta: ObjectCreateInfo,
    id: Option<id::GenericId>,
}

// Internal implementation of the object set
struct ObjectSetImpl {
    group: SynchronizationGroup,
    set_id: GlobalId,
    objects: Box<[ObjectData]>,
    allocation: AllocationMeta,
}

impl ObjectSetImpl {
    fn new(synchronization_group: SynchronizationGroup, objects: Box<[ObjectData]>, allocation: AllocationMeta) -> Self {
        Self{
            group: synchronization_group,
            set_id: GlobalId::new(),
            objects,
            allocation,
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

impl Drop for ObjectSetImpl {
    fn drop(&mut self) {
        self.group.get_manager().destroy_objects(self.objects.as_ref(), &self.allocation)
    }
}

// Needed because the SynchronizationSet mutex also protects the ObjectSet
unsafe impl Sync for ObjectSetImpl {
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
    fn new(synchronization_group: SynchronizationGroup, objects: Box<[ObjectData]>, allocation: AllocationMeta) -> Self {
        Self(Arc::new(ObjectSetImpl::new(synchronization_group, objects, allocation)))
    }

    pub fn get_set_id(&self) -> GlobalId {
        self.0.set_id
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