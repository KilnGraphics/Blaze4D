use std::any::Any;
use std::ptr::drop_in_place;
use std::sync::Mutex;
use ash::vk;

use crate::device::DeviceContext;

use crate::objects::{ObjectSet, SynchronizationGroup};
use crate::objects::buffer::{BufferDescription, BufferInstanceData, BufferViewDescription, BufferViewInstanceData};
use crate::objects::types::{BufferId, BufferViewId, GenericId, ImageId, ImageViewId, ObjectInstanceData, ObjectSetId, ObjectType, UnwrapToInstanceData};
use crate::objects::image::{ImageDescription, ImageInstanceData, ImageViewDescription, ImageViewInstanceData};
use crate::objects::allocator::{Allocation, AllocationError, AllocationStrategy};
use crate::objects::object_set::ObjectSetProvider;

/// Resource object sets are object sets specifically designed for resources that require backing
/// memory and synchronization. (i.e. Buffers, BufferViews etc.)
///
/// All objects of a resource object set have the same synchronization group.
///
/// All of the objects are only created when then [`ResourceObjectSetBuilder::build`] function is
/// called.
///
/// # Examples
///
/// ```
/// # use b4d_core::objects::buffer::BufferDescription;
/// # use b4d_core::objects::image::{ImageDescription, ImageViewDescription};
/// # use b4d_core::objects::resource_object_set::ResourceObjectSetBuilder;
/// # use b4d_core::objects::{Format, ImageSize, ImageSpec, SynchronizationGroup};
/// # let (_, device) = b4d_core::test::make_headless_instance_device();
/// use ash::vk;
///
/// // We need a synchronization group for our objects
/// let synchronization_group = SynchronizationGroup::new(device);
///
/// // Create a builder. It will use the synchronization group for all objects
/// let mut builder = ResourceObjectSetBuilder::new(synchronization_group);
///
/// // Add a request for a device only buffer. The buffer wont be created yet.
/// let buffer_id = builder.add_default_gpu_only_buffer(
///     BufferDescription::new_simple(1024, vk::BufferUsageFlags::VERTEX_BUFFER)
/// );
///
/// // Add a request for a device only image. Again the image wont be created just yet.
/// let image_id = builder.add_default_gpu_only_image(
///     ImageDescription::new_simple(
///         ImageSpec::new_single_sample(ImageSize::make_2d(128, 128), &Format::R8G8B8A8_SRGB),
///         vk::ImageUsageFlags::SAMPLED,
///     )
/// );
///
/// // We can add a image view for a previously requested image.
/// let image_view_id = builder.add_internal_image_view(
///     ImageViewDescription::make_full(
///         vk::ImageViewType::TYPE_2D,
///         &Format::R8G8B8A8_SRGB,
///         vk::ImageAspectFlags::COLOR
///     ),
///     image_id
/// );
///
/// // During the build call all of the objects will be created
/// let object_set = builder.build().unwrap();
///
/// // Now we can access the objects
/// let image_handle = unsafe { object_set.get_image_handle(image_id) };
///
/// // Or query information about them
/// let buffer_size = object_set.get_buffer_info(buffer_id).get_description().size;
///
/// // The objects will be destroyed when the object set is dropped. The object set type uses Arc
/// // internally so it can be cloned and the objects will only be dropped when all references
/// // have been dropped.
/// ```
/*pub struct ResourceObjectSetBuilder {
    set_id: ObjectSetId,
    device: DeviceContext,
    synchronization_group: SynchronizationGroup,
    requests: Vec<ResourceObjectCreateMetadata>,
}

impl ResourceObjectSetBuilder {
    /// Creates a new builder using the specified synchronization group
    ///
    /// The object set will use the device used for the synchronization group.
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
    pub fn add_default_gpu_only_buffer(&mut self, desc: BufferDescription) -> types::BufferId {
        let index = self.get_next_index();
        self.requests.push(ResourceObjectCreateMetadata::make_buffer(desc, AllocationStrategy::AutoGpuOnly, self.synchronization_group.clone()));

        types::BufferId::new(self.set_id, index)
    }

    /// Adds a request for a buffer that needs to be accessed by both the gpu and cpu.
    ///
    /// #Panics
    /// If there are more requests than the max object set size.
    pub fn add_default_gpu_cpu_buffer(&mut self, desc: BufferDescription) -> types::BufferId {
        let index = self.get_next_index();

        self.requests.push(ResourceObjectCreateMetadata::make_buffer(desc, AllocationStrategy::AutoGpuCpu, self.synchronization_group.clone()));

        types::BufferId::new(self.set_id, index)
    }

    /// Adds a buffer view request for a buffer that is created as part of this object set.
    ///
    /// #Panics
    /// If there are more requests than the max object set size or if the source buffer id does not
    /// map to a buffer.
    pub fn add_internal_buffer_view(&mut self, desc: BufferViewDescription, buffer: types::BufferId) -> types::BufferViewId {
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

        types::BufferViewId::new(self.set_id, index)
    }

    /// Adds a buffer view request for a buffer that is part of a different object set.
    ///
    /// #Panics
    /// If there are more requests than the max object set size or if the source buffer id is
    /// invalid.
    pub fn add_external_buffer_view(&mut self, desc: BufferViewDescription, set: ObjectSet, buffer: types::BufferId) -> types::BufferViewId {
        if buffer.get_set_id() != set.get_id() {
            panic!("Buffer set id does not match object set id");
        }
        let info = set.get_buffer_info(buffer).clone();

        let index = self.get_next_index();

        self.requests.push(ResourceObjectCreateMetadata::make_buffer_view(desc, Some(set), buffer, info));

        types::BufferViewId::new(self.set_id, index)
    }

    /// Adds a request for a image that only needs to be accessed by the gpu.
    ///
    /// #Panics
    /// If there are more requests than the max object set size.
    pub fn add_default_gpu_only_image(&mut self, desc: ImageDescription) -> types::ImageId {
        let index = self.get_next_index();

        self.requests.push(ResourceObjectCreateMetadata::make_image(desc, AllocationStrategy::AutoGpuOnly, self.synchronization_group.clone()));

        types::ImageId::new(self.set_id, index)
    }

    /// Adds a request for a image that needs to be accessed by both the gpu and cpu.
    ///
    /// #Panics
    /// If there are more requests than the max object set size.
    pub fn add_default_gpu_cpu_image(&mut self, desc: ImageDescription) -> types::ImageId {
        let index = self.get_next_index();

        self.requests.push(ResourceObjectCreateMetadata::make_image(desc, AllocationStrategy::AutoGpuCpu, self.synchronization_group.clone()));

        types::ImageId::new(self.set_id, index)
    }

    /// Adds a image view request for a image that is created as part of this object set.
    ///
    /// #Panics
    /// If there are more requests than the max object set size or if the source image id is
    /// invalid.
    pub fn add_internal_image_view(&mut self, desc: ImageViewDescription, image: types::ImageId) -> types::ImageViewId {
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

        types::ImageViewId::new(self.set_id, index)
    }

    /// Adds a image view request for a image that is part of a different object set.
    ///
    /// #Panics
    /// If there are more requests than the max object set size or if the source image id is
    /// invalid.
    pub fn add_external_image_view(&mut self, desc: ImageViewDescription, set: ObjectSet, image: types::ImageId) -> types::ImageViewId {
        if image.get_set_id() != set.get_id() {
            panic!("Buffer set id does not match object set id");
        }
        let info = set.get_image_info(image).clone();

        let index = self.get_next_index();

        self.requests.push(ResourceObjectCreateMetadata::make_image_view(desc, Some(set), image, info));

        types::ImageViewId::new(self.set_id, index)
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

struct BufferViewCreateMetadata {
    info: Box<BufferViewInfo>,
    buffer_set: Option<ObjectSet>,
    buffer_id: types::BufferId,
    handle: vk::BufferView,
}

impl BufferViewCreateMetadata {
    fn new(desc: BufferViewDescription, buffer_set: Option<ObjectSet>, buffer_id: types::BufferId, buffer_info: Arc<BufferInfo>) -> Self {
        Self {
            info: Box::new(BufferViewInfo::new(desc, buffer_id, buffer_info)),
            buffer_set,
            buffer_id,
            handle: vk::BufferView::null(),
        }
    }

    fn create(&mut self, device: &DeviceContext, split: &Splitter<ResourceObjectCreateMetadata>) -> Result<(), ObjectCreateError> {
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

struct ImageCreateMetadata {
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

struct ImageViewCreateMetadata {
    info: Box<ImageViewInfo>,
    image_set: Option<ObjectSet>,
    image_id: types::ImageId,
    handle: vk::ImageView,
}

impl ImageViewCreateMetadata {
    fn new(desc: ImageViewDescription, image_set: Option<ObjectSet>, image_id: types::ImageId, image_info: Arc<ImageInfo>) -> Self {
        Self {
            info: Box::new(ImageViewInfo::new(desc, image_id, image_info)),
            image_set,
            image_id,
            handle: vk::ImageView::null(),
        }
    }

    fn create(&mut self, device: &DeviceContext, split: &Splitter<ResourceObjectCreateMetadata>) -> Result<(), ObjectCreateError> {
        if self.handle == vk::ImageView::null() {
            let image = match self.image_set.as_ref() {
                Some(set) => {
                    unsafe { set.get_image_handle(self.image_id) }
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

enum ResourceObjectCreateMetadata {
    Buffer(BufferCreateMetadata),
    BufferView(BufferViewCreateMetadata),
    Image(ImageCreateMetadata),
    ImageView(ImageViewCreateMetadata),
}

impl ResourceObjectCreateMetadata {
    fn make_buffer(desc: BufferDescription, strategy: AllocationStrategy, group: SynchronizationGroup) -> Self {
        Self::Buffer(BufferCreateMetadata::new(desc, strategy, group))
    }

    fn make_buffer_view(desc: BufferViewDescription, buffer_set: Option<ObjectSet>, buffer_id: types::BufferId, buffer_info: Arc<BufferInfo>) -> Self {
        Self::BufferView(BufferViewCreateMetadata::new(desc, buffer_set, buffer_id, buffer_info))
    }

    fn make_image(desc: ImageDescription, strategy: AllocationStrategy, group: SynchronizationGroup) -> Self {
        Self::Image(ImageCreateMetadata::new(desc, strategy, group))
    }

    fn make_image_view(desc: ImageViewDescription, image_set: Option<ObjectSet>, image_id: types::ImageId, image_info: Arc<ImageInfo>) -> Self {
        Self::ImageView(ImageViewCreateMetadata::new(desc, image_set, image_id, image_info))
    }

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
}*/

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

pub struct ResourceObjectSet {
    set_id: ObjectSetId,
    device: DeviceContext,
    objects: Mutex<Objects>,
}

impl ResourceObjectSet {
    pub fn add_default_gpu_only_buffer(&self, desc: &BufferDescription, synchronization_group: SynchronizationGroup) -> BufferId {
        let (buffer, allocation) = unsafe { self.create_buffer(desc, AllocationStrategy::AutoGpuOnly) }.unwrap();
        self.insert_buffer(buffer, synchronization_group, allocation)
    }

    pub fn add_default_gpu_cpu_buffer(&self, desc: &BufferDescription, synchronization_group: SynchronizationGroup) -> BufferId {
        let (buffer, allocation) = unsafe { self.create_buffer(desc, AllocationStrategy::AutoGpuCpu) }.unwrap();
        self.insert_buffer(buffer, synchronization_group, allocation)
    }

    pub fn add_internal_buffer_view(&self, desc: &BufferViewDescription, source_id: BufferId) -> BufferViewId {
        if self.set_id != source_id.get_set_id() {
            panic!("source_id set id does not match object set id");
        }
        let source_data = self.try_get_buffer_data(source_id).unwrap();
        let buffer_view = unsafe { self.create_buffer_view(desc, source_data.get_handle()) }.unwrap();

        self.insert_buffer_view(buffer_view, source_data.get_synchronization_group().clone(), None)
    }

    pub fn add_external_buffer_view(&self, desc: &BufferViewDescription, source_id: BufferId, source_set: ObjectSet) -> BufferViewId {
        if self.set_id == source_set.get_id() {
            self.add_internal_buffer_view(desc, source_id)

        } else {
            let source_data = source_set.get_data(source_id);
            let buffer_view = unsafe { self.create_buffer_view(desc, source_data.get_handle()) }.unwrap();

            self.insert_buffer_view(buffer_view, source_data.get_synchronization_group().clone(), Some(source_set))
        }
    }

    pub fn add_default_gpu_only_image(&self, desc: &ImageDescription, synchronization_group: SynchronizationGroup) -> ImageId {
        let (image, allocation) = unsafe { self.create_image(desc, AllocationStrategy::AutoGpuOnly) }.unwrap();
        self.insert_image(image, synchronization_group, allocation)
    }

    pub fn add_default_gpu_cpu_image(&self, desc: &ImageDescription, synchronization_group: SynchronizationGroup) -> ImageId {
        let (image, allocation) = unsafe { self.create_image(desc, AllocationStrategy::AutoGpuCpu) }.unwrap();
        self.insert_image(image, synchronization_group, allocation)
    }

    pub fn add_internal_image_view(&self, desc: &ImageViewDescription, source_id: ImageId) -> ImageViewId {
        if self.set_id != source_id.get_set_id() {
            panic!("source_id set id does not match object set id");
        }
        let source_data = self.try_get_image_data(source_id).unwrap();
        let image_view = unsafe { self.create_image_view(desc, source_data.get_handle()) }.unwrap();

        self.insert_image_view(image_view, source_data.get_synchronization_group().clone(), None)
    }

    pub fn add_external_image_view(&self, desc: &ImageViewDescription, source_id: ImageId, source_set: ObjectSet) -> ImageViewId {
        if self.set_id == source_set.get_id() {
            self.add_internal_image_view(desc, source_id)

        } else {
            let source_data = source_set.get_data(source_id);
            let image_view = unsafe { self.create_image_view(desc, source_data.get_handle()) }.unwrap();

            self.insert_image_view(image_view, source_data.get_synchronization_group().clone(), Some(source_set))
        }
    }

    unsafe fn create_buffer(&self, desc: &BufferDescription, strategy: AllocationStrategy) -> Result<(vk::Buffer, Allocation), ObjectCreateError> {
        let create_info = vk::BufferCreateInfo::builder()
            .size(desc.size)
            .usage(desc.usage_flags)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let handle = self.device.vk().create_buffer(&create_info, None)?;

        let allocation = self.device.get_allocator().allocate_buffer_memory(handle, &strategy)?;

        self.device.vk().bind_buffer_memory(handle, allocation.memory(), allocation.offset())?;

        // TODO free resources on failure

        Ok((handle, allocation))
    }

    unsafe fn create_buffer_view(&self, desc: &BufferViewDescription, source: vk::Buffer) -> Result<vk::BufferView, ObjectCreateError> {
        let create_info = vk::BufferViewCreateInfo::builder()
            .buffer(source)
            .format(desc.format.get_format())
            .offset(desc.range.offset)
            .range(desc.range.length);

        let handle = self.device.vk().create_buffer_view(&create_info.build(), None)?;

        Ok(handle)
    }

    unsafe fn create_image(&self, desc: &ImageDescription, strategy: AllocationStrategy) -> Result<(vk::Image, Allocation), ObjectCreateError> {
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

        let handle = self.device.vk().create_image(&create_info.build(), None)?;

        let allocation = self.device.get_allocator().allocate_image_memory(handle, &strategy)?;

        self.device.vk().bind_image_memory(handle, allocation.memory(), allocation.offset())?;

        // TODO free resources on failure

        Ok((handle, allocation))
    }

    unsafe fn create_image_view(&self, desc: &ImageViewDescription, source: vk::Image) -> Result<vk::ImageView, ObjectCreateError> {
        let create_info = vk::ImageViewCreateInfo::builder()
            .image(source)
            .view_type(desc.view_type)
            .format(desc.format.get_format())
            .components(desc.components)
            .subresource_range(desc.subresource_range.as_vk_subresource_range());

        let handle = self.device.vk().create_image_view(&create_info, None)?;

        Ok(handle)
    }

    fn insert_buffer(&self, buffer: vk::Buffer, group: SynchronizationGroup, allocation: Allocation) -> BufferId {
        let index = {
            let mut guard = self.objects.lock().unwrap();
            guard.insert_buffer(buffer, group, allocation)
        };

        BufferId::new(self.set_id, index)
    }

    fn insert_buffer_view(&self, buffer_view: vk::BufferView, group: SynchronizationGroup, source_set: Option<ObjectSet>) -> BufferViewId {
        let index = {
            let mut guard = self.objects.lock().unwrap();
            guard.insert_buffer_view(buffer_view, group, source_set)
        };

        BufferViewId::new(self.set_id, index)
    }

    fn insert_image(&self, image: vk::Image, group: SynchronizationGroup, allocation: Allocation) -> ImageId {
        let index = {
            let mut guard = self.objects.lock().unwrap();
            guard.insert_image(image, group, allocation)
        };

        ImageId::new(self.set_id, index)
    }

    fn insert_image_view(&self, image_view: vk::ImageView, group: SynchronizationGroup, source_set: Option<ObjectSet>) -> ImageViewId {
        let index = {
            let mut guard = self.objects.lock().unwrap();
            guard.insert_image_view(image_view, group, source_set)
        };

        ImageViewId::new(self.set_id, index)
    }

    fn try_get_buffer_data(&self, id: BufferId) -> Option<&BufferInstanceData> {
        self.try_get_object_data(id.into()).map(|d| d.unwrap())
    }

    fn try_get_image_data(&self, id: ImageId) -> Option<&ImageInstanceData> {
        self.try_get_object_data(id.into()).map(|d| d.unwrap())
    }

    fn try_get_object_data(&self, id: GenericId) -> Option<ObjectInstanceData> {
        let index = id.get_index() as usize;
        let object_type = id.get_type();

        let guard = self.objects.lock().unwrap();
        unsafe { guard.objects.get(index)?.as_object_instance_data(object_type) }
    }
}

impl ObjectSetProvider for ResourceObjectSet {
    fn get_id(&self) -> ObjectSetId {
        self.set_id
    }

    fn get_object_data(&self, id: GenericId) -> ObjectInstanceData {
        self.try_get_object_data(id).unwrap()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Drop for ResourceObjectSet {
    fn drop(&mut self) {
        unsafe { self.objects.get_mut().unwrap().destroy(&self.device) };
    }
}

struct Objects {
    allocator: bumpalo::Bump,
    objects: Vec<Object>,
    allocations: Vec<Allocation>
}

impl Objects {
    unsafe fn destroy(&mut self, device: &DeviceContext) {
        // Need to destroy objects in reverse to account for potential dependencies
        let objects = std::mem::replace(&mut self.objects, Vec::new());
        for object in objects.into_iter() {
            object.destroy(device);
        }

        let device_allocator = device.get_allocator();
        let allocations = std::mem::replace(&mut self.allocations, Vec::new());
        for allocation in allocations.into_iter() {
            device_allocator.free(allocation);
        }
    }

    fn insert_buffer(&mut self, buffer: vk::Buffer, group: SynchronizationGroup, allocation: Allocation) -> u16 {
        let data = self.allocator.alloc(BufferInstanceData::new(buffer, group));
        let index = self.objects.len() as u16;

        self.objects.push(Object::Buffer(data));
        self.allocations.push(allocation);

        index
    }

    fn insert_buffer_view(&mut self, buffer_view: vk::BufferView, group: SynchronizationGroup, source_set: Option<ObjectSet>) -> u16 {
        let data = self.allocator.alloc(BufferViewInstanceData::new(buffer_view, group));
        let index = self.objects.len() as u16;

        self.objects.push(Object::BufferView(data, source_set));

        index
    }

    fn insert_image(&mut self, image: vk::Image, group: SynchronizationGroup, allocation: Allocation) -> u16 {
        let data = self.allocator.alloc(ImageInstanceData::new(image, group));
        let index = self.objects.len() as u16;

        self.objects.push(Object::Image(data));
        self.allocations.push(allocation);

        index
    }

    fn insert_image_view(&mut self, image_view: vk::ImageView, group: SynchronizationGroup, source_set: Option<ObjectSet>) -> u16 {
        let data = self.allocator.alloc(ImageViewInstanceData::new(image_view, group));
        let index = self.objects.len() as u16;

        self.objects.push(Object::ImageView(data, source_set));

        index
    }
}

enum Object {
    Buffer(*const BufferInstanceData),
    BufferView(*const BufferViewInstanceData, Option<ObjectSet>),
    Image(*const ImageInstanceData),
    ImageView(*const ImageViewInstanceData, Option<ObjectSet>),
}

impl Object {
    unsafe fn as_object_instance_data<'a>(&self, id_type: u8) -> Option<ObjectInstanceData<'a>> {
        // The to pointer conversion is necessary due to lifetimes
        match self {
            Self::Buffer(d, ..) => {
                if id_type != ObjectType::BUFFER {
                    return None;
                }
                Some(ObjectInstanceData::Buffer(d.as_ref().unwrap()))
            }
            Self::BufferView(d, ..) => {
                if id_type != ObjectType::BUFFER_VIEW {
                    return None;
                }
                Some(ObjectInstanceData::BufferView(d.as_ref().unwrap()))
            }
            Self::Image(d, ..) => {
                if id_type != ObjectType::IMAGE {
                    return None;
                }
                Some(ObjectInstanceData::Image(d.as_ref().unwrap()))
            }
            Self::ImageView(d, ..) => {
                if id_type != ObjectType::IMAGE_VIEW {
                    return None;
                }
                Some(ObjectInstanceData::ImageView(d.as_ref().unwrap()))
            }
        }
    }

    unsafe fn destroy(&self, device: &DeviceContext) {
        match self {
            Self::Buffer(d, ..) => {
                device.vk().destroy_buffer(d.as_ref().unwrap().get_handle(), None);
            }
            Self::BufferView(d, ..) => {
                device.vk().destroy_buffer_view(d.as_ref().unwrap().get_handle(), None);
            }
            Self::Image(d, ..) => {
                device.vk().destroy_image(d.as_ref().unwrap().get_handle(), None);
            }
            Self::ImageView(d, ..) => {
                device.vk().destroy_image_view(d.as_ref().unwrap().get_handle(), None);
            }
        }
    }
}

impl Drop for Object {
    fn drop(&mut self) {
        match self {
            Self::Buffer(d) => {
                unsafe { drop_in_place(*d as *mut BufferInstanceData) };
            }
            Self::BufferView(d, _) => {
                unsafe { drop_in_place(*d as *mut BufferViewInstanceData) };
            }
            Self::Image(d) => {
                unsafe { drop_in_place(*d as *mut ImageInstanceData) };
            }
            Self::ImageView(d, _) => {
                unsafe { drop_in_place(*d as *mut ImageViewInstanceData) };
            }
        }
    }
}