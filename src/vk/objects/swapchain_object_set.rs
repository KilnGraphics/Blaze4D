use std::any::Any;
use std::ptr::drop_in_place;
use std::sync::Mutex;
use ash::prelude::VkResult;
use ash::vk;
use crate::vk::objects::types::{GenericId, ImageId, ImageViewId, ObjectInstanceData, ObjectSetId, ObjectType, SurfaceId, SwapchainId, UnwrapToInstanceData};
use crate::vk::objects::image::{ImageInstanceData, ImageViewDescription, ImageViewInstanceData};
use crate::vk::objects::object_set::ObjectSetProvider;
use crate::vk::objects::swapchain::{SwapchainCreateDesc, SwapchainInstanceData};
use crate::vk::device::DeviceContext;

/// Swapchain object sets manage the creation of swapchains and have utilities for some common
/// objects needed for each image.
///
/// Derivative objects can be added in which case a object is created for each swapchain image.
/// ImageViews, binary Semaphores and Fences are currently supported as derivative objects.
///
/// The swapchain itself is created during the creation of the builder (this is necessary because
/// the builder needs to know the number of images that are in the swapchain). Just like with
/// resource object sets the derivative objects are only created during the
/// [`SwapchainObjectSetBuilder::build`] call.
///
/// # Examples
///
/// ```
/// ```
pub struct SwapchainObjectSet {
    set_id: ObjectSetId,
    device: DeviceContext,
    objects: Mutex<Objects>,
    image_ids: Box<[ImageId]>,
    source_surface: SurfaceId,
    swapchain_data: SwapchainInstanceData,
}

impl SwapchainObjectSet {
    pub fn new(device: DeviceContext, source_surface: SurfaceId, desc: &SwapchainCreateDesc) -> VkResult<Self> {
        let swapchain_fn = device.swapchain_khr().unwrap();

        let (surface, swapchain_info) = device.get_surface(source_surface).unwrap();
        let mut swapchain_info = swapchain_info.lock().unwrap();

        let old_swapchain = swapchain_info.get_current_handle().unwrap_or(vk::SwapchainKHR::null());

        let create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(desc.min_image_count)
            .image_format(desc.image_spec.format.get_format())
            .image_color_space(desc.image_spec.color_space)
            .image_extent(desc.image_spec.extent)
            .image_array_layers(desc.image_spec.array_layers)
            .image_usage(desc.usage)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(desc.pre_transform)
            .composite_alpha(desc.composite_alpha)
            .present_mode(desc.present_mode)
            .clipped(desc.clipped)
            .old_swapchain(old_swapchain);

        let new_swapchain = unsafe {
            swapchain_fn.create_swapchain(&create_info, None)
        }?;

        swapchain_info.set_swapchain(new_swapchain);

        let images = unsafe {
            swapchain_fn.get_swapchain_images(new_swapchain)

        }.map_err(|err| {
            // If there was an error destroy the swapchain and clear the surface swapchain info
            unsafe {
                swapchain_fn.destroy_swapchain(new_swapchain, None);
            }
            swapchain_info.clear();

            err
        })?;

        // Needed to keep this alive until we are done with all operations that could fail
        drop(swapchain_info);

        let images: Vec<_> = images.into_iter().map(|image| {
            image
        }).collect();

        let set_id = ObjectSetId::new();
        let image_ids: Vec<_> = (0u16..(images.len() as u16)).map(|index| ImageId::new(set_id, index)).collect();

        Ok(Self {
            set_id,
            device,
            objects: Mutex::new(Objects::new(images)),
            image_ids: image_ids.into_boxed_slice(),
            swapchain_data: SwapchainInstanceData::new(new_swapchain),
            source_surface,
        })
    }

    pub fn get_swapchain_id(&self) -> SwapchainId {
        SwapchainId::new(self.set_id, u16::MAX)
    }

    pub fn get_image_ids(&self) -> &[ImageId] {
        self.image_ids.as_ref()
    }

    pub fn add_image_views(&self, desc: &ImageViewDescription) -> Box<[ImageViewId]> {
        let mut image_views = Vec::with_capacity(self.image_ids.len());
        for image in self.image_ids.iter() {
            let source = self.try_get_image_data(*image).unwrap();
            let handle = unsafe { source.get_handle() };

            let image_view = unsafe { self.create_image_view(desc, handle) }.unwrap();
            image_views.push((image_view, *image))
        }

        let start_index = self.objects.lock().unwrap().insert_image_views(image_views);

        let mut ids = Vec::with_capacity(self.image_ids.len());
        for index in start_index..(start_index + (self.image_ids.len() as u16)) {
            ids.push(ImageViewId::new(self.set_id, index));
        }

        ids.into_boxed_slice()
    }

    unsafe fn create_image_view(&self, desc: &ImageViewDescription, source: vk::Image) -> VkResult<vk::ImageView> {
        let create_info = vk::ImageViewCreateInfo::builder()
            .image(source)
            .view_type(desc.view_type)
            .format(desc.format.get_format())
            .components(desc.components)
            .subresource_range(desc.subresource_range.as_vk_subresource_range());

        let handle = self.device.vk().create_image_view(&create_info, None)?;

        Ok(handle)
    }

    fn try_get_image_data(&self, id: ImageId) -> Option<&ImageInstanceData> {
        self.try_get_object_data(id.into()).map(|d| d.unwrap())
    }

    fn try_get_object_data(&self, id: GenericId) -> Option<ObjectInstanceData> {
        if id.get_type() == ObjectType::SWAPCHAIN && id.get_index() == u16::MAX {
            return Some(ObjectInstanceData::Swapchain(&self.swapchain_data));
        }

        let index = id.get_index() as usize;
        let object_type = id.get_type();

        let guard = self.objects.lock().unwrap();
        unsafe { guard.objects.get(index)?.as_object_instance_data(object_type) }
    }
}

impl ObjectSetProvider for SwapchainObjectSet {
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

impl Drop for SwapchainObjectSet {
    fn drop(&mut self) {
        unsafe { self.objects.get_mut().unwrap().destroy(&self.device) };

        let (_, swapchain_info) = self.device.get_surface(self.source_surface).unwrap();
        let mut swapchain_info = swapchain_info.lock().unwrap();

        unsafe { self.device.swapchain_khr().unwrap().destroy_swapchain(self.swapchain_data.get_handle(), None) };

        if let Some(current) = swapchain_info.get_current_handle() {
            if current == unsafe { self.swapchain_data.get_handle() } {
                swapchain_info.clear();
            }
        }
    }
}

struct Objects {
    allocator: bumpalo::Bump,
    objects: Vec<Object>,
}

impl Objects {
    fn new(images: Vec<vk::Image>) -> Self {
        let mut result = Self {
            allocator: bumpalo::Bump::new(),
            objects: Vec::with_capacity(images.len()),
        };

        for image in images {
            let data = result.allocator.alloc(ImageInstanceData::new(image));
            result.objects.push(Object::Image(data));
        }

        result
    }

    unsafe fn destroy(&mut self, device: &DeviceContext) {
        let objects = std::mem::replace(&mut self.objects, Vec::new());
        for object in objects.into_iter().rev() {
            object.destroy(device);
        }
    }

    fn insert_image_views(&mut self, image_views: Vec<(vk::ImageView, ImageId)>) -> u16 {
        let index = self.objects.len() as u16;

        self.objects.reserve(image_views.len());
        for (image_view, source_id) in image_views {
            let data = self.allocator.alloc(ImageViewInstanceData::new(image_view, source_id));
            self.objects.push(Object::ImageView(data));
        }

        index
    }
}

impl Drop for Objects {
    fn drop(&mut self) {
        if !self.objects.is_empty() {
            // This is fully in our control so this implies a bug insider the swapchain object set code
            panic!("Drop function for swapchain object set objects has been called while there are still objects inside");
        }
    }
}

enum Object {
    Image(*const ImageInstanceData),
    ImageView(*const ImageViewInstanceData),
}

impl Object {
    /// Creates a [`ObjectInstanceData`] for this object.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the assigned lifetime is smaller than the lifetime of this
    /// object.
    unsafe fn as_object_instance_data<'a>(&self, id_type: u8) -> Option<ObjectInstanceData<'a>> {
        match self {
            Object::Image(d) => {
                if id_type != ObjectType::IMAGE {
                    return None;
                }
                Some(ObjectInstanceData::Image(d.as_ref().unwrap()))
            }
            Object::ImageView(d) => {
                if id_type != ObjectType::IMAGE_VIEW {
                    return None;
                }
                Some(ObjectInstanceData::ImageView(d.as_ref().unwrap()))
            }
        }
    }

    unsafe fn destroy(&self, device: &DeviceContext) {
        match self {
            Object::Image(_) => {} // Images belong to the swapchain so nothing to do here
            Object::ImageView(d) => {
                device.vk().destroy_image_view(d.as_ref().unwrap().get_handle(), None);
            }
        }
    }
}

impl Drop for Object {
    fn drop(&mut self) {
        match self {
            Object::Image(d) => {
                unsafe { drop_in_place(*d as *mut ImageInstanceData) };
            }
            Object::ImageView(d) => {
                unsafe { drop_in_place(*d as *mut ImageViewInstanceData) };
            }
        }
    }
}