use std::any::Any;
use std::sync::Arc;
use ash::prelude::VkResult;
use ash::vk;
use ash::vk::{Buffer, BufferView, Image, ImageView, SwapchainKHR};
use crate::objects::{ObjectManager, ObjectSet, SynchronizationGroup};
use crate::objects::buffer::{BufferInfo, BufferViewInfo};
use crate::objects::id::{BufferId, BufferViewId, FenceId, ImageId, ImageViewId, ObjectSetId, SemaphoreId, SurfaceId, SwapchainId};
use crate::objects::image::{ImageDescription, ImageInfo, ImageViewDescription, ImageViewInfo};
use crate::objects::manager::ObjectSetProvider;
use crate::objects::swapchain::SwapchainCreateDesc;

struct ImageViewCreateMetadata {
    info: Box<ImageViewInfo>,
    handle: vk::ImageView,
}

struct BinarySemaphoreCreateMetadata {
    handle: vk::Semaphore,
}

struct FenceCreateMetadata {
    handle: vk::Fence,
}

enum DerivativeCreateMetadata {
    ImageView(ImageViewCreateMetadata),
    BinarySemaphore(BinarySemaphoreCreateMetadata),
    Fence(FenceCreateMetadata),
}

struct SwapchainImage {
    info: Arc<ImageInfo>,
    handle: vk::Image,
}

pub struct SwapchainObjectSetBuilder {
    manager: ObjectManager,
    set_id: ObjectSetId,
    surface: SurfaceId,
    swapchain: vk::SwapchainKHR,
    images: Box<[SwapchainImage]>,
    image_desc: ImageDescription,
    object_index: u16,
}

impl SwapchainObjectSetBuilder {
    pub(super) fn new(manager: ObjectManager, surface_id: SurfaceId, desc: SwapchainCreateDesc, synchronization_group: Option<SynchronizationGroup>) -> VkResult<Self> {
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

        // Need to keep this alive until we are done with all operations that could fail
        drop(swapchain_info);

        let image_desc = ImageDescription {
            spec: desc.image_spec.as_image_spec(),
            usage_flags: desc.usage,
        };

        let images : Box<_> = images.into_iter().map(|image| {
            let group = match &synchronization_group {
                None => manager.create_synchronization_group(),
                Some(group) => group.clone(),
            };

            SwapchainImage {
                info: Arc::new(ImageInfo::new(image_desc, group)),
                handle: image,
            }
        }).collect();

        // After this point errors are handled by the drop function of the SwapchainObjectSetBuilder
        Ok(Self {
            manager,
            set_id: ObjectSetId::new(),
            surface: surface_id,
            swapchain: new_swapchain,
            images,
            image_desc,
            object_index: 1,
        })
    }

    pub fn get_image_description(&self) -> &ImageDescription {
        &self.image_desc
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

    pub fn build(mut self) -> ObjectSet {
        // This is beyond ugly but necessary since we implement drop
        ObjectSet::new(SwapchainObjectSet {
            manager: self.manager.clone(),
            set_id: self.set_id,
            surface: self.surface,
            swapchain: std::mem::replace(&mut self.swapchain, vk::SwapchainKHR::null()),
        })
    }
}

impl Drop for SwapchainObjectSetBuilder {
    fn drop(&mut self) {
        if self.swapchain != vk::SwapchainKHR::null() {
            let device = &self.manager.0.device;
            let swapchain_fn = device.get_extension::<ash::extensions::khr::Swapchain>().unwrap();

            let surface = device.get_surface(self.surface).unwrap();
            let mut swapchain_info = surface.lock_swapchain_info();

            unsafe {
                swapchain_fn.destroy_swapchain(self.swapchain, None)
            };

            if swapchain_info.get_current_handle() == Some(self.swapchain) {
                swapchain_info.clear();
            }
            self.swapchain = vk::SwapchainKHR::null();
        }
    }
}

struct SwapchainObjectSet {
    manager: ObjectManager,
    set_id: ObjectSetId,
    surface: SurfaceId,
    swapchain: vk::SwapchainKHR,
}

impl SwapchainObjectSet {

}

impl ObjectSetProvider for SwapchainObjectSet {
    fn get_id(&self) -> ObjectSetId {
        self.set_id
    }

    fn get_swapchain_handle(&self, id: SwapchainId) -> SwapchainKHR {
        if id != SwapchainId::new(self.set_id, 0) {
            panic!("Invalid SwapchainId")
        }

        self.swapchain
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Drop for SwapchainObjectSet {
    fn drop(&mut self) {
        if self.swapchain != vk::SwapchainKHR::null() {
            let device = &self.manager.0.device;
            let swapchain_fn = device.get_extension::<ash::extensions::khr::Swapchain>().unwrap();

            let surface = device.get_surface(self.surface).unwrap();
            let mut swapchain_info = surface.lock_swapchain_info();

            unsafe {
                swapchain_fn.destroy_swapchain(self.swapchain, None)
            };

            if swapchain_info.get_current_handle() == Some(self.swapchain) {
                swapchain_info.clear();
            }
            self.swapchain = vk::SwapchainKHR::null();
        }
    }
}