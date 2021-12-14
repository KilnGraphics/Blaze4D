use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use crate::objects::buffer::{BufferCreateInfo, BufferViewCreateInfo};
use crate::objects::image::{ImageCreateMeta, ImageViewCreateMeta};
use crate::objects::id;
use crate::objects::manager::AllocationMeta;
use crate::objects::manager::synchronization_group::SynchronizationGroup;
use crate::util::id::GlobalId;

use ash::vk;
use ash::vk::Handle;

pub(super) enum ObjectData {
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
pub(super) enum ObjectCreateInfo {
    Buffer(BufferCreateInfo, gpu_allocator::MemoryLocation),
    InternalBufferView(BufferViewCreateInfo, usize),
    ExternalBufferView(BufferViewCreateInfo, ObjectSet, id::BufferId),
    Image(ImageCreateMeta, gpu_allocator::MemoryLocation),
    ImageView(ImageViewCreateMeta, usize),
    Event(),
}

pub struct ObjectSetBuilder {
    synchronization_group: SynchronizationGroup,
    set_id: GlobalId,
    requests: Vec<ObjectCreateInfo>,
}

impl ObjectSetBuilder {
    pub(super) fn new(synchronization_group: SynchronizationGroup) -> Self {
        Self {
            synchronization_group,
            set_id: GlobalId::new(),
            requests: Vec::new(),
        }
    }

    pub(super) fn with_capacity(synchronization_group: SynchronizationGroup, capacity: usize) -> Self {
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