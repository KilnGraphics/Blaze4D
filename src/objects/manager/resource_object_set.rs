use std::any::Any;
use ash::vk;
use ash::vk::Handle;
use winit::event::VirtualKeyCode::M;
use crate::device::DeviceContext;

use crate::objects::{id, ObjectManager, ObjectSet2, SynchronizationGroup};
use crate::objects::buffer::{BufferCreateDesc, BufferViewCreateDesc};
use crate::objects::id::{GenericId, ObjectSetId};
use crate::objects::image::{ImageCreateDesc, ImageViewCreateDesc};
use crate::objects::manager::allocator::{Allocation, AllocationError, AllocationStrategy, Allocator};
use crate::objects::manager::ObjectSetProvider;
use crate::util::slice_splitter::Splitter;

#[derive(Debug)]
pub enum ObjectCreateError {
    Vulkan(vk::Result),
    Allocation(AllocationError),
    InvalidReference,
}

impl<'s> From<ash::vk::Result> for ObjectCreateError {
    fn from(err: vk::Result) -> Self {
        ObjectCreateError::Vulkan(err)
    }
}

impl<'s> From<AllocationError> for ObjectCreateError {
    fn from(err: AllocationError) -> Self {
        ObjectCreateError::Allocation(err)
    }
}

pub(super) trait ResourceObjectCreator {
    fn create(&mut self, device: &DeviceContext, allocator: &Allocator, split: &Splitter<ResourceObjectCreateMetadata>) -> Result<(), ObjectCreateError>;

    fn abort(&mut self, device: &DeviceContext, allocator: &Allocator);

    fn reduce(self) -> (ResourceObjectData, Option<Allocation>);
}

pub(super) struct BufferCreateMetadata {
    desc: BufferCreateDesc,
    strategy: AllocationStrategy,
    handle: vk::Buffer,
    allocation: Option<Allocation>,
}

impl BufferCreateMetadata {
    fn new(desc: BufferCreateDesc, strategy: AllocationStrategy) -> Self {
        Self {
            desc,
            strategy,
            handle: vk::Buffer::null(),
            allocation: None,
        }
    }
}

impl ResourceObjectCreator for BufferCreateMetadata {
    fn create(&mut self, device: &DeviceContext, allocator: &Allocator, _: &Splitter<ResourceObjectCreateMetadata>) -> Result<(), ObjectCreateError> {
        if self.handle == vk::Buffer::null() {
            let create_info = vk::BufferCreateInfo::builder()
                .size(self.desc.size)
                .usage(self.desc.usage_flags)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            self.handle = unsafe {
                device.vk().create_buffer(&create_info, None)
            }?;
        }
        if self.allocation.is_none() {
            self.allocation = Some(allocator.allocate_buffer_memory(self.handle, &self.strategy)?);
            let alloc = self.allocation.as_ref().unwrap();

            unsafe {
                device.vk().bind_buffer_memory(self.handle, alloc.memory(), alloc.offset())
            }?;
        }
        Ok(())
    }

    fn abort(&mut self, device: &DeviceContext, allocator: &Allocator) {
        if self.handle != vk::Buffer::null() {
            unsafe { device.vk().destroy_buffer(self.handle, None) }
            self.handle = vk::Buffer::null();
        }
        match self.allocation.take() {
            Some(alloc) => {
                allocator.free(alloc);
            }
            None => {}
        }
    }

    fn reduce(self) -> (ResourceObjectData, Option<Allocation>) {
        if self.handle == vk::Buffer::null() || self.allocation.is_none() {
            panic!("Incomplete Buffer object")
        }

        let object = ResourceObjectData {
            object_type: ResourceObjectType::Buffer,
            handle: self.handle.as_raw(),
            source_set: None,
        };

        (object , self.allocation)
    }
}

pub(super) struct BufferViewCreateMetadata {
    desc: BufferViewCreateDesc,
    buffer_set: Option<ObjectSet2>,
    buffer_id: id::BufferId,
    handle: vk::BufferView,
}

impl BufferViewCreateMetadata {
    fn new(desc: BufferViewCreateDesc, buffer_set: Option<ObjectSet2>, buffer_id: id::BufferId) -> Self {
        Self {
            desc,
            buffer_set,
            buffer_id,
            handle: vk::BufferView::null(),
        }
    }
}

impl ResourceObjectCreator for BufferViewCreateMetadata {
    fn create(&mut self, device: &DeviceContext, _: &Allocator, split: &Splitter<ResourceObjectCreateMetadata>) -> Result<(), ObjectCreateError> {
        if self.handle == vk::BufferView::null() {
            let buffer = match self.buffer_set.as_ref() {
                Some(set) => {
                    set.get_handle(self.buffer_id)
                }
                None => {
                    let index = self.buffer_id.get_index() as usize;
                    match split.get(index).unwrap() {
                        ResourceObjectCreateMetadata::Buffer(BufferCreateMetadata{ handle, .. }) => *handle,
                        _ => return Err(ObjectCreateError::InvalidReference)
                    }
                }
            };

            let create_info = vk::BufferViewCreateInfo::builder()
                .buffer(buffer)
                .format(self.desc.format.get_format())
                .offset(self.desc.range.offset)
                .range(self.desc.range.length);

            self.handle = unsafe {
                device.vk().create_buffer_view(&create_info.build(), None)?
            }
        }
        Ok(())
    }

    fn abort(&mut self, device: &DeviceContext, _: &Allocator) {
        if self.handle != vk::BufferView::null() {
            unsafe { device.vk().destroy_buffer_view(self.handle, None) }
            self.handle = vk::BufferView::null()
        }
    }

    fn reduce(self) -> (ResourceObjectData, Option<Allocation>) {
        if self.handle == vk::BufferView::null() {
            panic!("Incomplete BufferView object")
        }

        let object = ResourceObjectData {
            object_type: ResourceObjectType::BufferView,
            handle: self.handle.as_raw(),
            source_set: self.buffer_set
        };

        (object, None)
    }
}

pub(super) struct ImageCreateMetadata {
    desc: ImageCreateDesc,
    strategy: AllocationStrategy,
    handle: vk::Image,
    allocation: Option<Allocation>,
}

impl ImageCreateMetadata {
    fn new(desc: ImageCreateDesc, strategy: AllocationStrategy) -> Self {
        Self {
            desc,
            strategy,
            handle: vk::Image::null(),
            allocation: None,
        }
    }
}

impl ResourceObjectCreator for ImageCreateMetadata {
    fn create(&mut self, device: &DeviceContext, allocator: &Allocator, _: &Splitter<ResourceObjectCreateMetadata>) -> Result<(), ObjectCreateError> {
        if self.handle == vk::Image::null() {
            let create_info = vk::ImageCreateInfo::builder()
                .image_type(self.desc.spec.size.get_vulkan_type())
                .format(self.desc.spec.format.get_format())
                .extent(self.desc.spec.size.as_extent_3d())
                .mip_levels(self.desc.spec.size.get_mip_levels())
                .array_layers(self.desc.spec.size.get_array_layers())
                .samples(self.desc.spec.sample_count)
                .tiling(vk::ImageTiling::OPTIMAL) // TODO we need some way to turn this linear
                .usage(self.desc.usage_flags)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            self.handle = unsafe {
                device.vk().create_image(&create_info.build(), None)
            }?;
        }
        if self.allocation.is_none() {
            self.allocation = Some(allocator.allocate_image_memory(self.handle, &self.strategy)?);
            let alloc = self.allocation.as_ref().unwrap();

            unsafe {
                device.vk().bind_image_memory(self.handle, alloc.memory(), alloc.offset())
            }?;
        }
        Ok(())
    }

    fn abort(&mut self, device: &DeviceContext, allocator: &Allocator) {
        if self.handle != vk::Image::null() {
            unsafe { device.vk().destroy_image(self.handle, None) }
            self.handle = vk::Image::null()
        }
        match self.allocation.take() {
            Some(alloc) => {
                allocator.free(alloc)
            }
            None => {}
        }
    }

    fn reduce(self) -> (ResourceObjectData, Option<Allocation>) {
        if self.handle == vk::Image::null() || self.allocation.is_none() {
            panic!("Incomplete Image object")
        }

        let object = ResourceObjectData {
            object_type: ResourceObjectType::Image,
            handle: self.handle.as_raw(),
            source_set: None
        };

        (object, self.allocation)
    }
}

pub(super) struct ImageViewCreateMetadata {
    desc: ImageViewCreateDesc,
    image_set: Option<ObjectSet2>,
    image_id: id::ImageId,
    handle: vk::ImageView,
}

impl ImageViewCreateMetadata {
    fn new(desc: ImageViewCreateDesc, image_set: Option<ObjectSet2>, image_id: id::ImageId) -> Self {
        Self {
            desc,
            image_set,
            image_id,
            handle: vk::ImageView::null(),
        }
    }
}

impl ResourceObjectCreator for ImageViewCreateMetadata {
    fn create(&mut self, device: &DeviceContext, _: &Allocator, split: &Splitter<ResourceObjectCreateMetadata>) -> Result<(), ObjectCreateError> {
        if self.handle == vk::ImageView::null() {
            let image = match self.image_set.as_ref() {
                Some(set) => {
                    set.get_handle(self.image_id)
                }
                None => {
                    let index = self.image_id.get_index() as usize;
                    match split.get(index).ok_or(ObjectCreateError::InvalidReference)? {
                        ResourceObjectCreateMetadata::Image(ImageCreateMetadata{ handle, .. }) => *handle,
                        _ => return Err(ObjectCreateError::InvalidReference)
                    }
                }
            };

            let create_info = vk::ImageViewCreateInfo::builder()
                .image(image)
                .view_type(self.desc.view_type)
                .format(self.desc.format.get_format())
                .components(self.desc.components)
                .subresource_range(self.desc.subresource_range.as_vk_subresource_range());

            self.handle = unsafe {
                device.vk().create_image_view(&create_info, None)?
            }
        }
        Ok(())
    }

    fn abort(&mut self, device: &DeviceContext, _: &Allocator) {
        if self.handle != vk::ImageView::null() {
            unsafe { device.vk().destroy_image_view(self.handle, None) }
            self.handle = vk::ImageView::null()
        }
    }

    fn reduce(self) -> (ResourceObjectData, Option<Allocation>) {
        if self.handle == vk::ImageView::null() {
            panic!("Incomplete ImageView object")
        }

        let object = ResourceObjectData {
            object_type: ResourceObjectType::ImageView,
            handle: self.handle.as_raw(),
            source_set: self.image_set
        };

        (object, None)
    }
}

pub(super) enum ResourceObjectCreateMetadata {
    Buffer(BufferCreateMetadata),
    BufferView(BufferViewCreateMetadata),
    Image(ImageCreateMetadata),
    ImageView(ImageViewCreateMetadata),
}

impl ResourceObjectCreateMetadata {
    fn make_buffer(desc: BufferCreateDesc, strategy: AllocationStrategy) -> Self {
        Self::Buffer(BufferCreateMetadata::new(desc, strategy))
    }

    fn make_buffer_view(desc: BufferViewCreateDesc, buffer_set: Option<ObjectSet2>, buffer_id: id::BufferId) -> Self {
        Self::BufferView(BufferViewCreateMetadata::new(desc, buffer_set, buffer_id))
    }

    fn make_image(desc: ImageCreateDesc, strategy: AllocationStrategy) -> Self {
        Self::Image(ImageCreateMetadata::new(desc, strategy))
    }

    fn make_image_view(desc: ImageViewCreateDesc, image_set: Option<ObjectSet2>, image_id: id::ImageId) -> Self {
        Self::ImageView(ImageViewCreateMetadata::new(desc, image_set, image_id))
    }
}

impl ResourceObjectCreator for ResourceObjectCreateMetadata {
    fn create(&mut self, device: &DeviceContext, allocator: &Allocator, split: &Splitter<ResourceObjectCreateMetadata>) -> Result<(), ObjectCreateError> {
        match self {
            ResourceObjectCreateMetadata::Buffer(data) => data.create(device, allocator, split),
            ResourceObjectCreateMetadata::BufferView(data) => data.create(device, allocator, split),
            ResourceObjectCreateMetadata::Image(data) => data.create(device, allocator, split),
            ResourceObjectCreateMetadata::ImageView(data) => data.create(device, allocator, split),
        }
    }

    fn abort(&mut self, device: &DeviceContext, allocator: &Allocator) {
        match self {
            ResourceObjectCreateMetadata::Buffer(data) => data.abort(device, allocator),
            ResourceObjectCreateMetadata::BufferView(data) => data.abort(device, allocator),
            ResourceObjectCreateMetadata::Image(data) => data.abort(device, allocator),
            ResourceObjectCreateMetadata::ImageView(data) => data.abort(device, allocator),
        }
    }

    fn reduce(self) -> (ResourceObjectData, Option<Allocation>) {
        match self {
            ResourceObjectCreateMetadata::Buffer(data) => data.reduce(),
            ResourceObjectCreateMetadata::BufferView(data) => data.reduce(),
            ResourceObjectCreateMetadata::Image(data) => data.reduce(),
            ResourceObjectCreateMetadata::ImageView(data) => data.reduce(),
        }
    }
}

pub(super) enum ResourceObjectType {
    Buffer,
    BufferView,
    Image,
    ImageView,
}

pub(super) struct ResourceObjectData {
    object_type: ResourceObjectType,
    handle: u64,
    source_set: Option<ObjectSet2>
}

impl ResourceObjectData {
    pub fn destroy(self, device: &DeviceContext) {
        match self.object_type {
            ResourceObjectType::Buffer => {
                let id = vk::Buffer::from_raw(self.handle);
                unsafe { device.vk().destroy_buffer(id, None) }
            }
            ResourceObjectType::BufferView => {
                let id = vk::BufferView::from_raw(self.handle);
                unsafe { device.vk().destroy_buffer_view(id, None) }
            }
            ResourceObjectType::Image => {
                let id = vk::Image::from_raw(self.handle);
                unsafe { device.vk().destroy_image(id, None) }
            }
            ResourceObjectType::ImageView => {
                let id = vk::ImageView::from_raw(self.handle);
                unsafe { device.vk().destroy_image_view(id, None) }
            }
        }
    }
}

pub struct ResourceObjectSetBuilder {
    set_id: ObjectSetId,
    manager: ObjectManager,
    synchronization_group: SynchronizationGroup,
    requests: Vec<ResourceObjectCreateMetadata>,
}

impl ResourceObjectSetBuilder {
    pub(super) fn new(synchronization_group: SynchronizationGroup) -> Self {
        let manager = synchronization_group.get_manager().clone();
        Self {
            synchronization_group,
            manager,
            set_id: ObjectSetId::new(),
            requests: Vec::new(),
        }
    }

    /// Returns the index of the next object.
    ///
    /// #Panics
    /// If the next index does not fit into a u16 number.
    fn get_next_index(&self) -> u16 {
        let index = self.requests.len();
        if index > u16::MAX as usize {
            panic!("Too many objects");
        }
        index as u16
    }

    /// Adds a request for a buffer that only needs to be accessed by the gpu.
    ///
    /// #Panics
    /// If there are more requests than the max object set size.
    pub fn add_default_gpu_only_buffer(&mut self, desc: BufferCreateDesc) -> id::BufferId {
        let index = self.get_next_index();
        self.requests.push(ResourceObjectCreateMetadata::make_buffer(desc, AllocationStrategy::AutoGpuOnly));

        id::BufferId::new(self.set_id, index)
    }

    /// Adds a request for a buffer that needs to be accessed by both the gpu and cpu.
    ///
    /// #Panics
    /// If there are more requests than the max object set size.
    pub fn add_default_gpu_cpu_buffer(&mut self, desc: BufferCreateDesc) -> id::BufferId {
        let index = self.get_next_index();

        self.requests.push(ResourceObjectCreateMetadata::make_buffer(desc, AllocationStrategy::AutoGpuCpu));

        id::BufferId::new(self.set_id, index)
    }

    /// Adds a buffer view request for a buffer that is created as part of this object set.
    ///
    /// #Panics
    /// If there are more requests than the max object set size.
    pub fn add_internal_buffer_view(&mut self, desc: BufferViewCreateDesc, buffer: id::BufferId) -> id::BufferViewId {
        if buffer.get_set_id() != self.set_id {
            panic!("Buffer set id does not match builder set id");
        }
        let index = self.get_next_index();

        self.requests.push(ResourceObjectCreateMetadata::make_buffer_view(desc, None, buffer));

        id::BufferViewId::new(self.set_id, index)
    }

    /// Adds a buffer view request for a buffer that is part of a different object set.
    ///
    /// #Panics
    /// If there are more requests than the max object set size.
    pub fn add_external_buffer_view(&mut self, desc: BufferViewCreateDesc, set: ObjectSet2, buffer: id::BufferId) -> id::BufferViewId {
        if buffer.get_set_id() != set.get_id() {
            panic!("Buffer set id does not match object set id");
        }
        let index = self.get_next_index();

        self.requests.push(ResourceObjectCreateMetadata::make_buffer_view(desc, Some(set), buffer));

        id::BufferViewId::new(self.set_id, index)
    }

    pub fn add_default_gpu_only_image(&mut self, desc: ImageCreateDesc) -> id::ImageId {
        let index = self.get_next_index();

        self.requests.push(ResourceObjectCreateMetadata::make_image(desc, AllocationStrategy::AutoGpuOnly));

        id::ImageId::new(self.set_id, index)
    }

    pub fn add_default_gpu_cpu_image(&mut self, desc: ImageCreateDesc) -> id::ImageId {
        let index = self.get_next_index();

        self.requests.push(ResourceObjectCreateMetadata::make_image(desc, AllocationStrategy::AutoGpuCpu));

        id::ImageId::new(self.set_id, index)
    }

    pub fn add_internal_image_view(&mut self, desc: ImageViewCreateDesc, image: id::ImageId) -> id::ImageViewId {
        if image.get_set_id() != self.set_id {
            panic!("Image set id does not match builder set id");
        }
        let index = self.get_next_index();

        self.requests.push(ResourceObjectCreateMetadata::make_image_view(desc, None, image));

        id::ImageViewId::new(self.set_id, index)
    }

    pub fn add_external_image_view(&mut self, desc: ImageViewCreateDesc, set: ObjectSet2, image: id::ImageId) -> id::ImageViewId {
        if image.get_set_id() != set.get_id() {
            panic!("Buffer set id does not match object set id");
        }
        let index = self.get_next_index();

        self.requests.push(ResourceObjectCreateMetadata::make_image_view(desc, Some(set), image));

        id::ImageViewId::new(self.set_id, index)
    }

    pub fn build(self) -> ObjectSet2 {
        let (objects, allocations) = self.manager.build_resource_objects(self.requests.into_boxed_slice());

        ObjectSet2::new(ResourceObjectSet {
            set_id: self.set_id,
            manager: self.manager,
            synchronization_group: self.synchronization_group,
            objects,
            allocations,
        })
    }
}

struct ResourceObjectSet {
    set_id: ObjectSetId,
    manager: ObjectManager,
    synchronization_group: SynchronizationGroup,
    objects: Box<[ResourceObjectData]>,
    allocations: Box<[Allocation]>
}

impl Drop for ResourceObjectSet {
    fn drop(&mut self) {
        let objects = std::mem::replace(&mut self.objects, Box::new([]));
        let allocations = std::mem::replace(&mut self.allocations, Box::new([]));

        self.manager.destroy_resource_objects(objects, allocations);
    }
}

impl ObjectSetProvider for ResourceObjectSet {
    fn get_id(&self) -> ObjectSetId {
        self.set_id
    }

    fn get_raw_handle(&self, id: GenericId) -> u64 {
        if id.get_set_id() != self.set_id {
            panic!("Id belongs to different object set")
        }

        let index = id.get_index() as usize;

        self.objects.get(index).unwrap().handle
    }

    fn get_synchronization_group(&self, _: GenericId) -> Option<SynchronizationGroup> {
        Some(self.synchronization_group.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}