use ash::prelude::VkResult;
use ash::vk;
use ash::vk::{ImageView, SwapchainKHR};
use crate::objects::ObjectManager;
use crate::objects::id::{FenceId, ImageId, ImageViewId, ObjectSetId, SemaphoreId, SurfaceId, SwapchainId};
use crate::objects::image::ImageViewDescription;
use crate::objects::swapchain::SwapchainCreateDesc;

pub struct SwapchainObjectSetBuilder {
    manager: ObjectManager,
    set_id: ObjectSetId,
    surface: SurfaceId,
    swapchain: vk::SwapchainKHR,
    images: Box<[vk::Image]>,
}

impl SwapchainObjectSetBuilder {
    fn new(manager: ObjectManager, surface_id: SurfaceId, desc: SwapchainCreateDesc) -> VkResult<Self> {
        let device = &manager.0.device;
        let swapchain_fn = device.get_extension::<ash::extensions::khr::Swapchain>().unwrap();

        let surface = device.get_surface(surface_id).unwrap();
        let mut swapchain_info = surface.lock_swapchain_info();

        let old_swapchain = swapchain_info.get_current_handle().unwrap_or(SwapchainKHR::null());

        let create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface.get_handle())
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
        drop(swapchain_info);

        let images = unsafe {
            swapchain_fn.get_swapchain_images(new_swapchain)
        }?;

        Ok(Self {
            manager,
            set_id: ObjectSetId::new(),
            surface: surface_id,
            swapchain: new_swapchain,
            images: images.into_boxed_slice(),
        })
    }

    pub fn get_swapchain_id(&self) -> SwapchainId {
        SwapchainId::new(self.set_id, 0)
    }

    pub fn get_image_ids(&self) -> Box<[ImageId]> {
        (0..self.images.len()).map(|index| ImageId::new(self.set_id, index as u16)).collect()
    }

    /// Adds a set of image views for each image of the swapchain
    pub fn add_views(&self, desc: ImageViewDescription) -> Box<[ImageViewId]> {
        todo!()
    }

    /// Adds a set of binary semaphores for each image of the swapchain
    pub fn add_binary_semaphores(&self) -> Box<[SemaphoreId]> {
        todo!()
    }

    /// Adds a set of fences for each image of the swapchain
    pub fn add_fences(&self) -> Box<[FenceId]> {
        todo!()
    }
}