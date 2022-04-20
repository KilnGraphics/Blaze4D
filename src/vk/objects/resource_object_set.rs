use std::any::Any;
use std::ptr::drop_in_place;
use std::sync::Mutex;
use ash::vk;

use crate::vk::device::DeviceContext;
use crate::vk::objects::{ObjectSet, SynchronizationGroup};
use crate::vk::objects::buffer::{BufferDescription, BufferInstanceData, BufferViewDescription, BufferViewInstanceData};
use crate::vk::objects::types::{BufferId, BufferViewId, GenericId, ImageId, ImageViewId, ObjectInstanceData, ObjectSetId, ObjectType, UnwrapToInstanceData};
use crate::vk::objects::image::{ImageDescription, ImageInstanceData, ImageViewDescription, ImageViewInstanceData};
use crate::vk::objects::allocator::{Allocation, AllocationError, AllocationStrategy};
use crate::vk::objects::object_set::ObjectSetProvider;

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
/// ```
pub struct ResourceObjectSet {
    set_id: ObjectSetId,
    device: DeviceContext,
    objects: Mutex<Objects>,
}

impl ResourceObjectSet {
    pub fn new(device: DeviceContext) -> Self {
        Self {
            set_id: ObjectSetId::new(),
            device,
            objects: Mutex::new(Objects::new()),
        }
    }

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
    fn new() -> Self {
        Self {
            allocator: bumpalo::Bump::new(),
            objects: Vec::new(),
            allocations: Vec::new(),
        }
    }

    /// Destroys all objects inside this set and frees all memory allocations. Any instance data for
    /// objects is dropped.
    ///
    /// # Safety
    ///
    /// The caller must ensure that there are no references to any instance data as they will be
    /// dropped. Currently this means this function **must only be called from inside
    /// [`ResourceObjectSet::drop`]**
    unsafe fn destroy(&mut self, device: &DeviceContext) {
        // Need to destroy objects in reverse to account for potential dependencies
        let objects = std::mem::replace(&mut self.objects, Vec::new());
        for object in objects.into_iter().rev() {
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

impl Drop for Objects {
    fn drop(&mut self) {
        if !self.objects.is_empty() {
            // This is fully in our control so this implies a bug insider the resource object set code
            panic!("Drop functions for resource object set objects has been called while there are still objects inside");
        }
    }
}

enum Object {
    Buffer(*const BufferInstanceData),
    BufferView(*const BufferViewInstanceData, Option<ObjectSet>),
    Image(*const ImageInstanceData),
    ImageView(*const ImageViewInstanceData, Option<ObjectSet>),
}

impl Object {
    /// Creates a [`ObjectInstanceData`] for this object.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the assigned lifetime is smaller than the lifetime of this
    /// object.
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

    /// Destroys the vulkan object. The instance data object is only dropped when this object is
    /// dropped.
    ///
    /// # Safety
    ///
    /// The instance object memory must be valid and this function must only be called once.
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

#[cfg(test)]
mod tests {
    use ash::vk::BufferUsageFlags;
    use super::*;

    #[test]
    fn test_buffer() {
        let (_, device) = crate::vk::test::make_headless_instance_device();

        let group = SynchronizationGroup::new(device.clone());
        let set = ResourceObjectSet::new(device);

        let buffer_desc = BufferDescription::new_simple(1024, BufferUsageFlags::TRANSFER_DST);
        let id = set.add_default_gpu_only_buffer(&buffer_desc, group.clone());

        let set = ObjectSet::new(set);

        assert_ne!(unsafe { set.get_data(id).get_handle() }, vk::Buffer::null());
        assert_eq!(set.get_data(id).get_synchronization_group(), &group);
    }
}