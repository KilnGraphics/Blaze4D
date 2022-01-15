use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::mem::ManuallyDrop;
use std::sync::Arc;
use crate::objects::buffer::{BufferCreateDesc, BufferViewCreateDesc};
use crate::objects::image::{ImageCreateDesc, ImageViewCreateDesc};
use crate::objects::{id, ObjectManager};
use crate::objects::manager::synchronization_group::SynchronizationGroup;

use ash::vk;
use ash::vk::Handle;
use crate::objects::id::ObjectSetId;
use crate::objects::manager::allocator::{Allocation, AllocationStrategy};
use crate::objects::manager::ObjectRequestDescription;

pub(super) enum ObjectData {
    Buffer{
        handle: vk::Buffer,
    },
    BufferView{
        handle: vk::BufferView,
        #[allow(unused)] // This is needed to prevent the source set from being destroyed early
        source_set: Option<ObjectSet>,
    },
    Image {
        handle: vk::Image,
    },
    ImageView {
        handle: vk::ImageView,
        #[allow(unused)] // This is needed to prevent the source set from being destroyed early
        source_set: Option<ObjectSet>,
    },
    BinarySemaphore {
        handle: vk::Semaphore,
    },
    TimelineSemaphore {
        handle: vk::Semaphore,
    },
    Event {
        handle: vk::Event,
    },
    Fence {
        handle: vk::Fence,
    },
}

impl ObjectData {
    fn get_raw_handle(&self) -> u64 {
        match self {
            ObjectData::Buffer { handle, .. } => handle.as_raw(),
            ObjectData::BufferView {handle, .. } => handle.as_raw(),
            ObjectData::Image { handle, .. } => handle.as_raw(),
            ObjectData::ImageView { handle, .. } => handle.as_raw(),
            ObjectData::BinarySemaphore { handle, .. } => handle.as_raw(),
            ObjectData::TimelineSemaphore { handle, .. } => handle.as_raw(),
            ObjectData::Event { handle, .. } => handle.as_raw(),
            ObjectData::Fence { handle, .. } => handle.as_raw(),
        }
    }
}

pub(super) struct ObjectSetData {
    pub objects: Box<[ObjectData]>,
    pub allocations: Box<[Allocation]>
}

/// Utility struct used to build an object set.
///
/// Collects information about objects that need to be created for an object set. The objects are
/// only created once the build method is called.
pub struct ObjectSetBuilder {
    synchronization_group: Option<SynchronizationGroup>,
    manager: ObjectManager,
    set_id: ObjectSetId,
    requests: Vec<ObjectRequestDescription>,
    requires_group: bool,
}

impl ObjectSetBuilder {
    pub(super) fn new(synchronization_group: SynchronizationGroup) -> Self {
        let manager = synchronization_group.get_manager().clone();
        Self {
            synchronization_group: Some(synchronization_group),
            manager,
            set_id: ObjectSetId::new(),
            requests: Vec::new(),
            requires_group: false,
        }
    }

    pub(super) fn new_no_group(manager: ObjectManager) -> Self {
        Self {
            synchronization_group: None,
            manager,
            set_id: ObjectSetId::new(),
            requests: Vec::new(),
            requires_group: false,
        }
    }

    /// Adds a request for a buffer that only needs to be accessed by the gpu
    pub fn add_default_gpu_only_buffer(&mut self, desc: BufferCreateDesc) -> id::BufferId {
        if self.synchronization_group.is_none() {
            panic!("Attempted to add buffer to object set without synchronization group");
        }
        self.requires_group = true;

        let index = self.validate_set_size();

        self.requests.push(ObjectRequestDescription::make_buffer(desc, AllocationStrategy::AutoGpuOnly));

        id::BufferId::new(self.set_id, index)
    }

    /// Adds a request for a buffer that needs to be accessed by both gpu and cpu
    pub fn add_default_gpu_cpu_buffer(&mut self, desc: BufferCreateDesc) -> id::BufferId {
        if self.synchronization_group.is_none() {
            panic!("Attempted to add buffer to object set without synchronization group");
        }
        self.requires_group = true;

        let index = self.validate_set_size();

        self.requests.push(ObjectRequestDescription::make_buffer(desc, AllocationStrategy::AutoGpuCpu));

        id::BufferId::new(self.set_id, index)
    }

    /// Adds a buffer view for a buffer created as part of this object set
    pub fn add_internal_buffer_view(&mut self, desc: BufferViewCreateDesc, buffer: id::BufferId) -> id::BufferViewId {
        if self.synchronization_group.is_none() {
            panic!("Attempted to add buffer view to object set without synchronization group");
        }
        self.requires_group = true;

        if buffer.get_set_id() != self.set_id {
            panic!("Buffer global id does not match set id")
        }
        let index = self.validate_set_size();

        self.requests.push(ObjectRequestDescription::make_buffer_view(desc, None, buffer));

        id::BufferViewId::new(self.set_id, index)
    }

    /// Adds a buffer view for a buffer owned by a different object set
    pub fn add_external_buffer_view(&mut self, desc: BufferViewCreateDesc, set: ObjectSet, buffer: id::BufferId) -> id::BufferViewId {
        if self.synchronization_group.is_none() {
            panic!("Attempted to add buffer view to object set without synchronization group");
        }
        self.requires_group = true;

        if buffer.get_set_id() != set.get_set_id() {
            panic!("Buffer global id does not match set id")
        }

        if set.get_synchronization_group().unwrap() != self.synchronization_group.as_ref().unwrap() {
            panic!("Buffer does not match internal synchronization group")
        }

        let index = self.validate_set_size();

        self.requests.push(ObjectRequestDescription::make_buffer_view(desc, Some(set), buffer));

        id::BufferViewId::new(self.set_id, index)
    }

    /// Adds a request for a image that only needs to be accessed by the gpu
    pub fn add_default_gpu_only_image(&mut self, desc: ImageCreateDesc) -> id::ImageId {
        if self.synchronization_group.is_none() {
            panic!("Attempted to add image to object set without synchronization group");
        }
        self.requires_group = true;

        let index = self.validate_set_size();

        self.requests.push(ObjectRequestDescription::make_image(desc, AllocationStrategy::AutoGpuOnly));

        id::ImageId::new(self.set_id, index)
    }

    /// Adds a request for a image that needs to be accessed by both gpu and cpu
    pub fn add_default_gpu_cpu_image(&mut self, desc: ImageCreateDesc) -> id::ImageId {
        if self.synchronization_group.is_none() {
            panic!("Attempted to add image to object set without synchronization group");
        }
        self.requires_group = true;

        let index = self.validate_set_size();

        self.requests.push(ObjectRequestDescription::make_image(desc, AllocationStrategy::AutoGpuCpu));

        id::ImageId::new(self.set_id, index)
    }

    /// Adds a image view for a image created as part of this object set
    pub fn add_internal_image_view(&mut self, desc: ImageViewCreateDesc, image: id::ImageId) -> id::ImageViewId {
        if self.synchronization_group.is_none() {
            panic!("Attempted to add image view to object set without synchronization group");
        }
        self.requires_group = true;

        if image.get_set_id() != self.set_id {
            panic!("Image global id does not match set id")
        }
        let index = self.validate_set_size();

        self.requests.push(ObjectRequestDescription::make_image_view(desc, None, image));

        id::ImageViewId::new(self.set_id, index)
    }

    /// Adds a image view for a image owned by a different object set
    pub fn add_external_image_view(&mut self, desc: ImageViewCreateDesc, set: ObjectSet, image: id::ImageId) -> id::ImageViewId {
        if self.synchronization_group.is_none() {
            panic!("Attempted to add image view to object set without synchronization group");
        }
        self.requires_group = true;

        if image.get_set_id() != set.get_set_id() {
            panic!("Image global id does not match set id")
        }

        if set.get_synchronization_group().unwrap() != self.synchronization_group.as_ref().unwrap() {
            panic!("Image does not match internal synchronization group")
        }

        let index = self.validate_set_size();

        self.requests.push(ObjectRequestDescription::make_image_view(desc, Some(set), image));

        id::ImageViewId::new(self.set_id, index)
    }

    pub fn add_binary_semaphore(&mut self) -> id::BinarySemaphoreId {
        let index = self.validate_set_size();

        self.requests.push(ObjectRequestDescription::make_binary_semaphore());

        id::BinarySemaphoreId::new(self.set_id, index)
    }

    pub fn add_timeline_semaphore(&mut self, initial_value: u64) -> id::TimelineSemaphoreId {
        let index = self.validate_set_size();

        self.requests.push(ObjectRequestDescription::make_timeline_semaphore(initial_value));

        id::TimelineSemaphoreId::new(self.set_id, index)
    }

    pub fn add_event(&mut self) -> id::EventId {
        let index = self.validate_set_size();

        self.requests.push(ObjectRequestDescription::make_event(false));

        id::EventId::new(self.set_id, index)
    }

    pub fn add_fence(&mut self, initial_signaled: bool) -> id::FenceId {
        let index = self.validate_set_size();

        self.requests.push(ObjectRequestDescription::make_fence(initial_signaled));

        id::FenceId::new(self.set_id, index)
    }

    /// Creates the objects and returns the resulting object set
    pub fn build(self) -> ObjectSet {
        let group = if self.requires_group { self.synchronization_group } else { None };

        let (objects, allocation) = self.manager.create_objects(self.requests.as_slice());
        ObjectSet::new(self.set_id, group, self.manager, objects, allocation)
    }

    /// Ensures that there is enough space to add another object to the set and returns the index of
    /// the next object
    fn validate_set_size(&self) -> u16 {
        let index = self.requests.len();
        if index > u16::MAX as usize {
            panic!("Too many objects")
        }
        index as u16
    }
}

// Internal implementation of the object set
struct ObjectSetImpl {
    group: Option<SynchronizationGroup>,
    manager: ObjectManager,
    set_id: ObjectSetId,

    // Screw unwrap
    data: ManuallyDrop<ObjectSetData>,
}

impl ObjectSetImpl {
    fn new(set_id: ObjectSetId, synchronization_group: Option<SynchronizationGroup>, manager: ObjectManager, objects: Box<[ObjectData]>, allocations: Box<[Allocation]>) -> Self {
        Self{
            group: synchronization_group,
            manager,
            set_id,
            data: ManuallyDrop::new(ObjectSetData {
                objects,
                allocations,
            })
        }
    }

    fn get_raw_handle(&self, id: id::GenericId) -> Option<u64> {
        if id.get_set_id() != self.set_id {
            return None;
        }

        // Invalid local id but matching global is a serious error
        Some(self.data.objects.get(id.get_index() as usize).unwrap().get_raw_handle())
    }

    fn get_buffer_handle(&self, id: id::BufferId) -> Option<vk::Buffer> {
        if id.get_set_id() != self.set_id {
            return None;
        }

        // Invalid local id but matching global is a serious error
        match self.data.objects.get(id.get_index() as usize).unwrap() {
            ObjectData::Buffer { handle, .. } => Some(*handle),
            _ => panic!("Object type mismatch"),
        }
    }

    fn get_buffer_view_handle(&self, id: id::BufferViewId) -> Option<vk::BufferView> {
        if id.get_set_id()!= self.set_id {
            return None;
        }

        // Invalid local id but matching global is a serious error
        match self.data.objects.get(id.get_index() as usize).unwrap() {
            ObjectData::BufferView { handle, .. } => Some(*handle),
            _ => panic!("Object type mismatch"),
        }
    }

    fn get_image_handle(&self, id: id::ImageId) -> Option<vk::Image> {
        if id.get_set_id() != self.set_id {
            return None;
        }

        // Invalid local id but matching global is a serious error
        match self.data.objects.get(id.get_index() as usize).unwrap() {
            ObjectData::Image { handle, .. } => Some(*handle),
            _ => panic!("Object type mismatch"),
        }
    }

    fn get_image_view_handle(&self, id: id::ImageViewId) -> Option<vk::ImageView> {
        if id.get_set_id() != self.set_id {
            return None;
        }

        // Invalid local id but matching global is a serious error
        match self.data.objects.get(id.get_index() as usize).unwrap() {
            ObjectData::ImageView { handle, .. } => Some(*handle),
            _ => panic!("Object type mismatch"),
        }
    }

    fn get_binary_semaphore_handle(&self, id: id::BinarySemaphoreId) -> Option<vk::Semaphore> {
        if id.get_set_id() != self.set_id {
            return None;
        }

        // Invalid local id but matching global is a serious error
        match self.data.objects.get(id.get_index() as usize).unwrap() {
            ObjectData::BinarySemaphore { handle, .. } => Some(*handle),
            _ => panic!("Object type mismatch"),
        }
    }

    fn get_timeline_semaphore_handle(&self, id: id::TimelineSemaphoreId) -> Option<vk::Semaphore> {
        if id.get_set_id() != self.set_id {
            return None;
        }

        // Invalid local id but matching global is a serious error
        match self.data.objects.get(id.get_index() as usize).unwrap() {
            ObjectData::TimelineSemaphore { handle, .. } => Some(*handle),
            _ => panic!("Object type mismatch"),
        }
    }

    fn get_event_handle(&self, id: id::EventId) -> Option<vk::Event> {
        if id.get_set_id() != self.set_id {
            return None;
        }

        // Invalid local id but matching global is a serious error
        match self.data.objects.get(id.get_index() as usize).unwrap() {
            ObjectData::Event { handle, .. } => Some(*handle),
            _ => panic!("Object type mismatch"),
        }
    }

    fn get_fence_handle(&self, id: id::FenceId) -> Option<vk::Fence> {
        if id.get_set_id() != self.set_id {
            return None;
        }

        // Invalid local id but matching global is a serious error
        match self.data.objects.get(id.get_index() as usize).unwrap() {
            ObjectData::Fence { handle, .. } => Some(*handle),
            _ => panic!("Object type mismatch"),
        }
    }
}

impl Drop for ObjectSetImpl {
    fn drop(&mut self) {
        let data = unsafe { ManuallyDrop::take(&mut self.data) };
        self.manager.destroy_objects(data.objects, data.allocations);
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
    fn new(set_id: ObjectSetId, synchronization_group: Option<SynchronizationGroup>, manager: ObjectManager, objects: Box<[ObjectData]>, allocations: Box<[Allocation]>) -> Self {
        Self(Arc::new(ObjectSetImpl::new(set_id, synchronization_group, manager, objects, allocations)))
    }

    pub fn get_set_id(&self) -> ObjectSetId {
        self.0.set_id
    }

    /// Returns the synchronization group that controls access to this object set.
    pub fn get_synchronization_group(&self) -> Option<&SynchronizationGroup> {
        self.0.group.as_ref()
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

    /// Returns the handle of a buffer view that is part of this object set.
    ///
    /// If the id is not part of the object set (i.e. the global id does not match) None will be
    /// returned. If the id is invalid (matching global id but local id is invalid or object type
    /// is not a buffer view) the function panics.
    pub fn get_buffer_view_handle(&self, id: id::BufferViewId) -> Option<vk::BufferView> {
        self.0.get_buffer_view_handle(id)
    }

    /// Returns the handle of a image that is part of this object set.
    ///
    /// If the id is not part of the object set (i.e. the global id does not match) None will be
    /// returned. If the id is invalid (matching global id but local id is invalid or object type
    /// is not a image) the function panics.
    pub fn get_image_handle(&self, id: id::ImageId) -> Option<vk::Image> {
        self.0.get_image_handle(id)
    }

    /// Returns the handle of a image view that is part of this object set.
    ///
    /// If the id is not part of the object set (i.e. the global id does not match) None will be
    /// returned. If the id is invalid (matching global id but local id is invalid or object type
    /// is not a image view) the function panics.
    pub fn get_image_view_handle(&self, id: id::ImageViewId) -> Option<vk::ImageView> {
        self.0.get_image_view_handle(id)
    }

    pub fn get_binary_semaphore_handle(&self, id: id::BinarySemaphoreId) -> Option<vk::Semaphore> {
        self.0.get_binary_semaphore_handle(id)
    }

    pub fn get_timeline_semaphore_handle(&self, id: id::TimelineSemaphoreId) -> Option<vk::Semaphore> {
        self.0.get_timeline_semaphore_handle(id)
    }

    pub fn get_event_handle(&self, id: id::EventId) -> Option<vk::Event> {
        self.0.get_event_handle(id)
    }

    pub fn get_fence_handle(&self, id: id::FenceId) -> Option<vk::Fence> {
        self.0.get_fence_handle(id)
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