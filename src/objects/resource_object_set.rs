use std::any::Any;
use std::sync::Arc;
use ash::vk;
use crate::device::DeviceContext;

use crate::objects::{id, ObjectSet, SynchronizationGroup};
use crate::objects::buffer::{BufferDescription, BufferInfo, BufferViewDescription, BufferViewInfo};
use crate::objects::id::{BufferId, BufferViewId, ImageId, ImageViewId, ObjectSetId};
use crate::objects::image::{ImageDescription, ImageInfo, ImageViewDescription, ImageViewInfo};
use crate::objects::allocator::{Allocation, AllocationError, AllocationStrategy};
use crate::objects::object_set::ObjectSetProvider;
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

pub(in crate::objects) trait ResourceObjectCreator {
    fn create(&mut self, device: &DeviceContext, split: &Splitter<ResourceObjectCreateMetadata>) -> Result<(), ObjectCreateError>;

    fn abort(&mut self, device: &DeviceContext);

    fn reduce(self) -> (ResourceObjectData, Option<Allocation>);
}

pub(in crate::objects) struct BufferCreateMetadata {
    info: Arc<BufferInfo>,
    strategy: AllocationStrategy,
    handle: vk::Buffer,
    allocation: Option<Allocation>,
}

impl BufferCreateMetadata {
    fn new(desc: BufferDescription, strategy: AllocationStrategy, group: SynchronizationGroup) -> Self {
        Self {
            info: Arc::new(BufferInfo::new(desc, group)),
            strategy,
            handle: vk::Buffer::null(),
            allocation: None,
        }
    }
}

impl ResourceObjectCreator for BufferCreateMetadata {
    fn create(&mut self, device: &DeviceContext, _: &Splitter<ResourceObjectCreateMetadata>) -> Result<(), ObjectCreateError> {
        if self.handle == vk::Buffer::null() {
            let desc = self.info.get_description();
            let create_info = vk::BufferCreateInfo::builder()
                .size(desc.size)
                .usage(desc.usage_flags)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            self.handle = unsafe {
                device.vk().create_buffer(&create_info, None)
            }?;
        }
        if self.allocation.is_none() {
            self.allocation = Some(device.get_allocator().allocate_buffer_memory(self.handle, &self.strategy)?);
            let alloc = self.allocation.as_ref().unwrap();

            unsafe {
                device.vk().bind_buffer_memory(self.handle, alloc.memory(), alloc.offset())
            }?;
        }
        Ok(())
    }

    fn abort(&mut self, device: &DeviceContext) {
        if self.handle != vk::Buffer::null() {
            unsafe { device.vk().destroy_buffer(self.handle, None) }
            self.handle = vk::Buffer::null();
        }
        match self.allocation.take() {
            Some(alloc) => {
                device.get_allocator().free(alloc);
            }
            None => {}
        }
    }

    fn reduce(self) -> (ResourceObjectData, Option<Allocation>) {
        if self.handle == vk::Buffer::null() || self.allocation.is_none() {
            panic!("Incomplete Buffer object")
        }

        let object = ResourceObjectData::Buffer {
            handle: self.handle,
            info: self.info,
        };

        (object , self.allocation)
    }
}

pub(in crate::objects) struct BufferViewCreateMetadata {
    info: Box<BufferViewInfo>,
    buffer_set: Option<ObjectSet>,
    buffer_id: id::BufferId,
    handle: vk::BufferView,
}

impl BufferViewCreateMetadata {
    fn new(desc: BufferViewDescription, buffer_set: Option<ObjectSet>, buffer_id: id::BufferId, buffer_info: Arc<BufferInfo>) -> Self {
        Self {
            info: Box::new(BufferViewInfo::new(desc, buffer_id, buffer_info)),
            buffer_set,
            buffer_id,
            handle: vk::BufferView::null(),
        }
    }
}

impl ResourceObjectCreator for BufferViewCreateMetadata {
    fn create(&mut self, device: &DeviceContext, split: &Splitter<ResourceObjectCreateMetadata>) -> Result<(), ObjectCreateError> {
        if self.handle == vk::BufferView::null() {
            let buffer = match self.buffer_set.as_ref() {
                Some(set) => {
                    set.get_buffer_handle(self.buffer_id)
                }
                None => {
                    let index = self.buffer_id.get_index() as usize;
                    match split.get(index).unwrap() {
                        ResourceObjectCreateMetadata::Buffer(BufferCreateMetadata{ handle, .. }) => *handle,
                        _ => return Err(ObjectCreateError::InvalidReference)
                    }
                }
            };

            let desc = self.info.get_description();
            let create_info = vk::BufferViewCreateInfo::builder()
                .buffer(buffer)
                .format(desc.format.get_format())
                .offset(desc.range.offset)
                .range(desc.range.length);

            self.handle = unsafe {
                device.vk().create_buffer_view(&create_info.build(), None)?
            }
        }
        Ok(())
    }

    fn abort(&mut self, device: &DeviceContext) {
        if self.handle != vk::BufferView::null() {
            unsafe { device.vk().destroy_buffer_view(self.handle, None) }
            self.handle = vk::BufferView::null()
        }
    }

    fn reduce(self) -> (ResourceObjectData, Option<Allocation>) {
        if self.handle == vk::BufferView::null() {
            panic!("Incomplete BufferView object")
        }

        let object = ResourceObjectData::BufferView {
            handle: self.handle,
            info: self.info,
            source_set: self.buffer_set,
        };

        (object, None)
    }
}

pub(in crate::objects) struct ImageCreateMetadata {
    info: Arc<ImageInfo>,
    strategy: AllocationStrategy,
    handle: vk::Image,
    allocation: Option<Allocation>,
}

impl ImageCreateMetadata {
    fn new(desc: ImageDescription, strategy: AllocationStrategy, group: SynchronizationGroup) -> Self {
        Self {
            info: Arc::new(ImageInfo::new(desc, group)),
            strategy,
            handle: vk::Image::null(),
            allocation: None,
        }
    }
}

impl ResourceObjectCreator for ImageCreateMetadata {
    fn create(&mut self, device: &DeviceContext, _: &Splitter<ResourceObjectCreateMetadata>) -> Result<(), ObjectCreateError> {
        if self.handle == vk::Image::null() {
            let desc = self.info.get_description();
            let create_info = vk::ImageCreateInfo::builder()
                .image_type(desc.spec.size.get_vulkan_type())
                .format(desc.spec.format.get_format())
                .extent(desc.spec.size.as_extent_3d())
                .mip_levels(desc.spec.size.get_mip_levels())
                .array_layers(desc.spec.size.get_array_layers())
                .samples(desc.spec.sample_count)
                .tiling(vk::ImageTiling::OPTIMAL) // TODO we need some way to turn this linear
                .usage(desc.usage_flags)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            self.handle = unsafe {
                device.vk().create_image(&create_info.build(), None)
            }?;
        }
        if self.allocation.is_none() {
            self.allocation = Some(device.get_allocator().allocate_image_memory(self.handle, &self.strategy)?);
            let alloc = self.allocation.as_ref().unwrap();

            unsafe {
                device.vk().bind_image_memory(self.handle, alloc.memory(), alloc.offset())
            }?;
        }
        Ok(())
    }

    fn abort(&mut self, device: &DeviceContext) {
        if self.handle != vk::Image::null() {
            unsafe { device.vk().destroy_image(self.handle, None) }
            self.handle = vk::Image::null()
        }
        match self.allocation.take() {
            Some(alloc) => {
                device.get_allocator().free(alloc)
            }
            None => {}
        }
    }

    fn reduce(self) -> (ResourceObjectData, Option<Allocation>) {
        if self.handle == vk::Image::null() || self.allocation.is_none() {
            panic!("Incomplete Image object")
        }

        let object = ResourceObjectData::Image {
            handle: self.handle,
            info: self.info
        };

        (object, self.allocation)
    }
}

pub(in crate::objects) struct ImageViewCreateMetadata {
    info: Box<ImageViewInfo>,
    image_set: Option<ObjectSet>,
    image_id: id::ImageId,
    handle: vk::ImageView,
}

impl ImageViewCreateMetadata {
    fn new(desc: ImageViewDescription, image_set: Option<ObjectSet>, image_id: id::ImageId, image_info: Arc<ImageInfo>) -> Self {
        Self {
            info: Box::new(ImageViewInfo::new(desc, image_id, image_info)),
            image_set,
            image_id,
            handle: vk::ImageView::null(),
        }
    }
}

impl ResourceObjectCreator for ImageViewCreateMetadata {
    fn create(&mut self, device: &DeviceContext, split: &Splitter<ResourceObjectCreateMetadata>) -> Result<(), ObjectCreateError> {
        if self.handle == vk::ImageView::null() {
            let image = match self.image_set.as_ref() {
                Some(set) => {
                    set.get_image_handle(self.image_id)
                }
                None => {
                    let index = self.image_id.get_index() as usize;
                    match split.get(index).ok_or(ObjectCreateError::InvalidReference)? {
                        ResourceObjectCreateMetadata::Image(ImageCreateMetadata{ handle, .. }) => *handle,
                        _ => return Err(ObjectCreateError::InvalidReference)
                    }
                }
            };

            let desc = self.info.get_description();
            let create_info = vk::ImageViewCreateInfo::builder()
                .image(image)
                .view_type(desc.view_type)
                .format(desc.format.get_format())
                .components(desc.components)
                .subresource_range(desc.subresource_range.as_vk_subresource_range());

            self.handle = unsafe {
                device.vk().create_image_view(&create_info, None)?
            }
        }
        Ok(())
    }

    fn abort(&mut self, device: &DeviceContext) {
        if self.handle != vk::ImageView::null() {
            unsafe { device.vk().destroy_image_view(self.handle, None) }
            self.handle = vk::ImageView::null()
        }
    }

    fn reduce(self) -> (ResourceObjectData, Option<Allocation>) {
        if self.handle == vk::ImageView::null() {
            panic!("Incomplete ImageView object")
        }

        let object = ResourceObjectData::ImageView {
            handle: self.handle,
            info: self.info,
            source_set: self.image_set
        };

        (object, None)
    }
}

pub(in crate::objects) enum ResourceObjectCreateMetadata {
    Buffer(BufferCreateMetadata),
    BufferView(BufferViewCreateMetadata),
    Image(ImageCreateMetadata),
    ImageView(ImageViewCreateMetadata),
}

impl ResourceObjectCreateMetadata {
    fn make_buffer(desc: BufferDescription, strategy: AllocationStrategy, group: SynchronizationGroup) -> Self {
        Self::Buffer(BufferCreateMetadata::new(desc, strategy, group))
    }

    fn make_buffer_view(desc: BufferViewDescription, buffer_set: Option<ObjectSet>, buffer_id: id::BufferId, buffer_info: Arc<BufferInfo>) -> Self {
        Self::BufferView(BufferViewCreateMetadata::new(desc, buffer_set, buffer_id, buffer_info))
    }

    fn make_image(desc: ImageDescription, strategy: AllocationStrategy, group: SynchronizationGroup) -> Self {
        Self::Image(ImageCreateMetadata::new(desc, strategy, group))
    }

    fn make_image_view(desc: ImageViewDescription, image_set: Option<ObjectSet>, image_id: id::ImageId, image_info: Arc<ImageInfo>) -> Self {
        Self::ImageView(ImageViewCreateMetadata::new(desc, image_set, image_id, image_info))
    }
}

impl ResourceObjectCreator for ResourceObjectCreateMetadata {
    fn create(&mut self, device: &DeviceContext, split: &Splitter<ResourceObjectCreateMetadata>) -> Result<(), ObjectCreateError> {
        match self {
            ResourceObjectCreateMetadata::Buffer(data) => data.create(device, split),
            ResourceObjectCreateMetadata::BufferView(data) => data.create(device, split),
            ResourceObjectCreateMetadata::Image(data) => data.create(device, split),
            ResourceObjectCreateMetadata::ImageView(data) => data.create(device, split),
        }
    }

    fn abort(&mut self, device: &DeviceContext) {
        match self {
            ResourceObjectCreateMetadata::Buffer(data) => data.abort(device),
            ResourceObjectCreateMetadata::BufferView(data) => data.abort(device),
            ResourceObjectCreateMetadata::Image(data) => data.abort(device),
            ResourceObjectCreateMetadata::ImageView(data) => data.abort(device),
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

pub(in crate::objects) enum ResourceObjectData {
    Buffer {
        handle: vk::Buffer,
        info: Arc<BufferInfo>,
    },
    BufferView {
        handle: vk::BufferView,
        info: Box<BufferViewInfo>,
        source_set: Option<ObjectSet>,
    },
    Image {
        handle: vk::Image,
        info: Arc<ImageInfo>,
    },
    ImageView {
        handle: vk::ImageView,
        info: Box<ImageViewInfo>,
        source_set: Option<ObjectSet>,
    }
}

impl ResourceObjectData {
    pub fn destroy(self, device: &DeviceContext) {
        match self {
            ResourceObjectData::Buffer{ handle, .. } => {
                unsafe { device.vk().destroy_buffer(handle, None) }
            }
            ResourceObjectData::BufferView{ handle, source_set, .. } => {
                unsafe { device.vk().destroy_buffer_view(handle, None) }
                drop(source_set); // Keep it alive until here
            }
            ResourceObjectData::Image{ handle, .. } => {
                unsafe { device.vk().destroy_image(handle, None) }
            }
            ResourceObjectData::ImageView{ handle, source_set, .. } => {
                unsafe { device.vk().destroy_image_view(handle, None) }
                drop(source_set); // Keep it alive until here
            }
        }
    }
}

/// Resource object sets are object sets specifically designed for resources that require backing
/// memory and synchronization. (i.e. Buffers, BufferViews etc.)
///
/// All objects of a resource object set have the same synchronization group.
///
/// All of the objects are only created when then [`ResourceObjectSetBuilder::build`] function is
/// called.
pub struct ResourceObjectSetBuilder {
    set_id: ObjectSetId,
    device: DeviceContext,
    synchronization_group: SynchronizationGroup,
    requests: Vec<ResourceObjectCreateMetadata>,
}

impl ResourceObjectSetBuilder {
    pub fn new(synchronization_group: SynchronizationGroup) -> Self {
        let device = synchronization_group.get_device().clone();
        Self {
            synchronization_group,
            device,
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
    pub fn add_default_gpu_only_buffer(&mut self, desc: BufferDescription) -> id::BufferId {
        let index = self.get_next_index();
        self.requests.push(ResourceObjectCreateMetadata::make_buffer(desc, AllocationStrategy::AutoGpuOnly, self.synchronization_group.clone()));

        id::BufferId::new(self.set_id, index)
    }

    /// Adds a request for a buffer that needs to be accessed by both the gpu and cpu.
    ///
    /// #Panics
    /// If there are more requests than the max object set size.
    pub fn add_default_gpu_cpu_buffer(&mut self, desc: BufferDescription) -> id::BufferId {
        let index = self.get_next_index();

        self.requests.push(ResourceObjectCreateMetadata::make_buffer(desc, AllocationStrategy::AutoGpuCpu, self.synchronization_group.clone()));

        id::BufferId::new(self.set_id, index)
    }

    /// Adds a buffer view request for a buffer that is created as part of this object set.
    ///
    /// #Panics
    /// If there are more requests than the max object set size or if the source buffer id does not
    /// map to a buffer.
    pub fn add_internal_buffer_view(&mut self, desc: BufferViewDescription, buffer: id::BufferId) -> id::BufferViewId {
        if buffer.get_set_id() != self.set_id {
            panic!("Buffer set id does not match builder set id");
        }
        let info = match self.requests.get(buffer.get_index() as usize).unwrap() {
            ResourceObjectCreateMetadata::Buffer(buff) => {
                buff.info.clone()
            }
            _ => panic!("Buffer id does not map to a buffer")
        };

        let index = self.get_next_index();

        self.requests.push(ResourceObjectCreateMetadata::make_buffer_view(desc, None, buffer, info));

        id::BufferViewId::new(self.set_id, index)
    }

    /// Adds a buffer view request for a buffer that is part of a different object set.
    ///
    /// #Panics
    /// If there are more requests than the max object set size or if the source buffer id is
    /// invalid.
    pub fn add_external_buffer_view(&mut self, desc: BufferViewDescription, set: ObjectSet, buffer: id::BufferId) -> id::BufferViewId {
        if buffer.get_set_id() != set.get_id() {
            panic!("Buffer set id does not match object set id");
        }
        let info = set.get_buffer_info(buffer).clone();

        let index = self.get_next_index();

        self.requests.push(ResourceObjectCreateMetadata::make_buffer_view(desc, Some(set), buffer, info));

        id::BufferViewId::new(self.set_id, index)
    }

    /// Adds a request for a image that only needs to be accessed by the gpu.
    ///
    /// #Panics
    /// If there are more requests than the max object set size.
    pub fn add_default_gpu_only_image(&mut self, desc: ImageDescription) -> id::ImageId {
        let index = self.get_next_index();

        self.requests.push(ResourceObjectCreateMetadata::make_image(desc, AllocationStrategy::AutoGpuOnly, self.synchronization_group.clone()));

        id::ImageId::new(self.set_id, index)
    }

    /// Adds a request for a image that needs to be accessed by both the gpu and cpu.
    ///
    /// #Panics
    /// If there are more requests than the max object set size.
    pub fn add_default_gpu_cpu_image(&mut self, desc: ImageDescription) -> id::ImageId {
        let index = self.get_next_index();

        self.requests.push(ResourceObjectCreateMetadata::make_image(desc, AllocationStrategy::AutoGpuCpu, self.synchronization_group.clone()));

        id::ImageId::new(self.set_id, index)
    }

    /// Adds a image view request for a image that is created as part of this object set.
    ///
    /// #Panics
    /// If there are more requests than the max object set size or if the source image id is
    /// invalid.
    pub fn add_internal_image_view(&mut self, desc: ImageViewDescription, image: id::ImageId) -> id::ImageViewId {
        if image.get_set_id() != self.set_id {
            panic!("Image set id does not match builder set id");
        }
        let info = match self.requests.get(image.get_index() as usize).unwrap() {
            ResourceObjectCreateMetadata::Image(img) => {
                img.info.clone()
            }
            _ => panic!("Image id does not map to a image")
        };

        let index = self.get_next_index();

        self.requests.push(ResourceObjectCreateMetadata::make_image_view(desc, None, image, info));

        id::ImageViewId::new(self.set_id, index)
    }

    /// Adds a image view request for a image that is part of a different object set.
    ///
    /// #Panics
    /// If there are more requests than the max object set size or if the source image id is
    /// invalid.
    pub fn add_external_image_view(&mut self, desc: ImageViewDescription, set: ObjectSet, image: id::ImageId) -> id::ImageViewId {
        if image.get_set_id() != set.get_id() {
            panic!("Buffer set id does not match object set id");
        }
        let info = set.get_image_info(image).clone();

        let index = self.get_next_index();

        self.requests.push(ResourceObjectCreateMetadata::make_image_view(desc, Some(set), image, info));

        id::ImageViewId::new(self.set_id, index)
    }

    fn create_objects(&mut self) -> Result<(), ObjectCreateError> {
        let slice = self.requests.as_mut_slice();

        for i in 0..slice.len() {
            let (splitter, elem) = Splitter::new(slice, i);
            elem.create(&self.device, &splitter)?;
        }

        Ok(())
    }

    fn destroy_objects(&mut self) {
        for request in self.requests.iter_mut().rev() {
            request.abort(&self.device)
        }
    }

    /// Creates all objects and returns the completed object set.
    pub fn build(mut self) -> Result<ObjectSet, ObjectCreateError> {
        if let Err(error) = self.create_objects() {
            self.destroy_objects();
            return Err(error)
        }

        let mut allocations = Vec::new();
        let mut objects = Vec::with_capacity(self.requests.len());

        for request in self.requests {
            let (object, allocation) = request.reduce();
            objects.push(object);

            if let Some(allocation) = allocation {
                allocations.push(allocation)
            }
        }

        Ok(ObjectSet::new(ResourceObjectSet {
            set_id: self.set_id,
            device: self.device,
            objects: objects.into_boxed_slice(),
            allocations: allocations.into_boxed_slice(),
        }))
    }
}

struct ResourceObjectSet {
    set_id: ObjectSetId,
    device: DeviceContext,
    objects: Box<[ResourceObjectData]>,
    allocations: Box<[Allocation]>
}

impl Drop for ResourceObjectSet {
    fn drop(&mut self) {
        let objects = std::mem::replace(&mut self.objects, Box::new([]));
        let allocations = std::mem::replace(&mut self.allocations, Box::new([]));

        for object in objects.into_vec().into_iter().rev() {
            object.destroy(&self.device)
        }

        let allocator = self.device.get_allocator();
        for allocation in allocations.into_vec() {
            allocator.free(allocation);
        }
    }
}

impl ObjectSetProvider for ResourceObjectSet {
    fn get_id(&self) -> ObjectSetId {
        self.set_id
    }

    fn get_buffer_handle(&self, id: BufferId) -> vk::Buffer {
        match self.objects.get(id.get_index() as usize).unwrap() {
            ResourceObjectData::Buffer { handle, .. } => *handle,
            _ => panic!("Id does not map to buffer")
        }
    }

    fn get_buffer_info(&self, id: BufferId) -> &Arc<BufferInfo> {
        match self.objects.get(id.get_index() as usize).unwrap() {
            ResourceObjectData::Buffer { info, .. } => info,
            _ => panic!("Id does not map to buffer")
        }
    }

    fn get_buffer_view_handle(&self, id: BufferViewId) -> vk::BufferView {
        match self.objects.get(id.get_index() as usize).unwrap() {
            ResourceObjectData::BufferView { handle, .. } => *handle,
            _ => panic!("Id does not map to buffer view")
        }
    }

    fn get_buffer_view_info(&self, id: BufferViewId) -> &BufferViewInfo {
        match self.objects.get(id.get_index() as usize).unwrap() {
            ResourceObjectData::BufferView { info, .. } => info.as_ref(),
            _ => panic!("Id does not map to buffer view")
        }
    }

    fn get_image_handle(&self, id: ImageId) -> vk::Image {
        match self.objects.get(id.get_index() as usize).unwrap() {
            ResourceObjectData::Image { handle, .. } => *handle,
            _ => panic!("Id does not map to image")
        }
    }

    fn get_image_info(&self, id: ImageId) -> &Arc<ImageInfo> {
        match self.objects.get(id.get_index() as usize).unwrap() {
            ResourceObjectData::Image { info, .. } => info,
            _ => panic!("Id does not map to image")
        }
    }

    fn get_image_view_handle(&self, id: ImageViewId) -> vk::ImageView {
        match self.objects.get(id.get_index() as usize).unwrap() {
            ResourceObjectData::ImageView { handle, .. } => *handle,
            _ => panic!("Id does not map to image view")
        }
    }

    fn get_image_view_info(&self, id: ImageViewId) -> &ImageViewInfo {
        match self.objects.get(id.get_index() as usize).unwrap() {
            ResourceObjectData::ImageView { info, .. } => info.as_ref(),
            _ => panic!("Id does not map to image view")
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}