use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use ash::prelude::VkResult;
use ash::vk;
use crate::prelude::*;
use crate::vk::device::VkQueue;

struct SwapchainManagerImpl {
    device: DeviceContext,
    surface: vk::SurfaceKHR,
    required_usage_flags: vk::ImageUsageFlags,
    format_priorities: Box<[vk::Format]>,

    /// This mutex additionally protects synchronized operations on the surface as well as any
    /// created swapchain.
    swapchain_mutex: Mutex<Option<vk::SwapchainKHR>>,
    swapchain_fn: ash::extensions::khr::Swapchain,
    surface_fn: ash::extensions::khr::Surface,
}

impl SwapchainManagerImpl {
    pub fn new(device: DeviceContext, surface: vk::SurfaceKHR) -> Self {
        let surface_fn = device.get_instance().surface_khr().unwrap().clone();
        let swapchain_fn = device.swapchain_khr().unwrap().clone();

        Self {
            device,
            surface,
            required_usage_flags: vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST,
            format_priorities: Box::new([vk::Format::R8G8B8A8_SRGB]),
            swapchain_mutex: Mutex::new(None),
            swapchain_fn,
            surface_fn,
        }
    }

    fn try_rebuild(&self, size: Vec2u32) -> VkResult<Option<vk::SwapchainKHR>> {
        let device = *self.device.get_physical_device();

        let formats = unsafe {
            self.surface_fn.get_physical_device_surface_formats(device, self.surface)
        }?;
        let format;
        if let Some(format_opt) = self.find_best_format(formats.as_slice()) {
            format = format_opt;
        } else {
            return Ok(None);
        }

        let capabilities = unsafe {
            self.surface_fn.get_physical_device_surface_capabilities(device, self.surface)
        }?;
        if !Self::is_in_extent(&capabilities, size) {
            return Ok(None);
        }
        let min_image_count = std::cmp::max(capabilities.min_image_count, 3);

        let mut info = vk::SwapchainCreateInfoKHR::builder()
            .surface(self.surface)
            .min_image_count(min_image_count)
            .image_format(format.format)
            .image_color_space(format.color_space)
            .image_extent(vk::Extent2D{ width: size[0], height: size[1] })
            .image_array_layers(1)
            .image_usage(self.required_usage_flags)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(todo!())
            .composite_alpha(todo!())
            .present_mode(vk::PresentModeKHR::FIFO)
            .clipped(true);

        let guard = self.swapchain_mutex.lock().unwrap();
        if let Some(old_swapcahin) = guard.as_ref() {
            info = info.old_swapchain(*old_swapcahin);
        }

        let new_swapchain = unsafe {
            self.swapchain_fn.create_swapchain(&info, None)
        }?;

        *guard = Some(new_swapchain);
        drop(guard);

        Ok(Some(new_swapchain))
    }

    fn find_best_format(&self, available: &[vk::SurfaceFormatKHR]) -> Option<vk::SurfaceFormatKHR> {
        let formats: HashSet<vk::Format> = available.iter().filter_map(|entry| {
            if entry.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR {
                Some(entry.format)
            } else {
                None
            }
        }).collect();

        for option in self.format_priorities.iter() {
            if formats.contains(option) {
                return Some(vk::SurfaceFormatKHR {
                    format: *option,
                    color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR
                });
            }
        }
        None
    }

    fn is_in_extent(capabilities: &vk::SurfaceCapabilitiesKHR, extent: Vec2u32) -> bool {
        capabilities.min_image_extent.width <= extent[0] &&
            capabilities.min_image_extent.height <= extent[1] &&
            capabilities.max_image_extent.width >= extent[0] &&
            capabilities.max_image_extent.height >= extent[1]
    }
}

#[derive(Clone)]
pub struct SwapchainManager(Arc<SwapchainManagerImpl>);

impl SwapchainManager {
    pub fn new(device: DeviceContext, surface: vk::SurfaceKHR) -> Self {
        Self(Arc::new(SwapchainManagerImpl::new(device, surface)))
    }

    pub fn try_rebuild(&self, size: Vec2u32) -> VkResult<Option<SwapchainInstance>> {
        if let Some(swapchain) = self.0.try_rebuild(size)? {
            // TODO handle an error here cleanly?
            let images = unsafe { self.0.swapchain_fn.get_swapchain_images(swapchain) }.unwrap();

            Ok(Some(SwapchainInstance {
                manager: self.0.clone(),
                swapchain,
                images: images.into_boxed_slice()
            }))
        } else {
            Ok(None)
        }
    }
}

pub struct SwapchainInstance {
    manager: Arc<SwapchainManagerImpl>,
    swapchain: vk::SwapchainKHR,
    images: Box<[vk::Image]>,
}

impl SwapchainInstance {
    pub fn get_images(&self) -> &[vk::Image] {
        self.images.as_ref()
    }

    pub unsafe fn acquire_next_image(&self, timeout: u64, semaphore: vk::Semaphore, fence: vk::Fence) -> VkResult<(u32, bool)> {
        let guard = self.manager.swapchain_mutex.lock().unwrap();
        let result = self.manager.swapchain_fn.acquire_next_image(self.swapchain, timeout, semaphore, fence);
        drop(guard);
        result
    }

    pub unsafe fn queue_present(&self, queue: VkQueue, wait_semaphores: &[vk::Semaphore], index: u32) -> VkResult<bool> {
        let info = vk::PresentInfoKHR::builder()
            .wait_semaphores(wait_semaphores)
            .swapchains(std::slice::from_ref(&self.swapchain))
            .image_indices(std::slice::from_ref(&index));

        let guard = self.manager.swapchain_mutex.lock().unwrap();
        let queue = queue.lock_queue();
        let result = self.manager.swapchain_fn.queue_present(*queue, &info);
        drop(queue);
        drop(guard);
        result
    }
}

impl Drop for SwapchainInstance {
    fn drop(&mut self) {
        let mut guard = self.manager.swapchain_mutex.lock().unwrap();
        unsafe { self.manager.swapchain_fn.destroy_swapchain(self.swapchain, None) };
        if let Some(current_swapchain) = guard.as_ref() {
            if *current_swapchain == self.swapchain {
                *guard = None;
            }
        }
    }
}