use std::fmt::{Debug, Formatter};
use std::ops::{BitAnd, BitOr};
use std::sync::{Arc, Mutex, MutexGuard, Weak};

use ash::prelude::VkResult;
use ash::vk;
use ash::vk::{Flags, Handle};

use crate::objects::id::{ImageId, ObjectId};
use crate::objects::ObjectSetProvider;
use crate::vk::objects::surface::SurfaceProvider;

use crate::prelude::*;

pub struct DeviceSurface {
    device: Arc<DeviceContext>,
    weak: Weak<DeviceSurface>,
    #[allow(unused)] // Just here to keep the provider alive
    surface_provider: Box<dyn SurfaceProvider>,
    surface: vk::SurfaceKHR,
    present_queue_families: Box<[bool]>,

    /// The current swapchain info.
    ///
    /// If both the swapchain mutex and the info mutex must be lock simultaneously (for example during
    /// creation and destruction) then the info mutex **must always** be lock first to avoid a deadlock.
    current_swapchain: Mutex<SurfaceSwapchainInfo>,
}

impl DeviceSurface {
    pub(super) fn new(device: Arc<DeviceContext>, surface: Box<dyn SurfaceProvider>, weak: Weak<DeviceSurface>, present_families: Box<[bool]>) -> Self {
        Self {
            device: device.clone(),
            weak,
            surface: surface.get_handle().unwrap(),
            surface_provider: surface,
            present_queue_families: present_families,
            current_swapchain: Mutex::new(SurfaceSwapchainInfo::new())
        }
    }

    pub fn get_surface_present_modes(&self) -> VkResult<Vec<vk::PresentModeKHR>> {
        unsafe {
            self.device.get_instance().surface_khr().unwrap().get_physical_device_surface_present_modes(*self.device.get_physical_device(), self.surface)
        }
    }

    pub fn get_surface_capabilities(&self) -> VkResult<vk::SurfaceCapabilitiesKHR> {
        unsafe {
            self.device.get_instance().surface_khr().unwrap().get_physical_device_surface_capabilities(*self.device.get_physical_device(), self.surface)
        }
    }

    pub fn get_surface_formats(&self) -> VkResult<Vec<vk::SurfaceFormatKHR>> {
        unsafe {
            self.device.get_instance().surface_khr().unwrap().get_physical_device_surface_formats(*self.device.get_physical_device(), self.surface)
        }
    }

    /// Returns the queue family present support for this surface.
    ///
    /// If the n-th entry of the slice is true then queue family n supports presentation to this
    /// surface.
    pub fn get_present_support(&self) -> &[bool] {
        self.present_queue_families.as_ref()
    }

    /// Returns true if the specified queue family supports presentation to this surface.
    pub fn is_present_supported(&self, family: u32) -> bool {
        *self.present_queue_families.get(family as usize).unwrap()
    }

    /// Creates a swapchain from a [`SwapchainConfig`].
    ///
    /// On some implementations (for example Wayland) the current extent field of the surface capabilities
    /// may be 0 which means it cannot be used to determine the desired extent of the swapchain. As
    /// such calling code should use some platform dependant way to determine the desired extent.
    ///
    /// If the current surface capabilities report a max extent of 0 [`SwapchainCreateError::NoExtent`]
    /// is returned.
    ///
    /// If some part of the config is not supported by the surface [`SwapchainCreateError::Unsupported`]
    /// is returned.
    pub fn create_swapchain(&self, config: &SwapchainConfig, extent: Vec2u32) -> Result<Arc<SurfaceSwapchain>, SwapchainCreateError> {
        let capabilities = self.get_surface_capabilities()?;

        let format = self.find_best_format(&config)?;

        let mut info = vk::SwapchainCreateInfoKHR::builder()
            .min_image_count(self.find_best_image_count(&capabilities, &config)?)
            .image_format(format.format)
            .image_color_space(format.color_space)
            .image_extent(self.validate_extent(&capabilities, extent)?)
            .image_array_layers(1)
            .image_usage(self.find_best_usage_flags(&capabilities, &config)?)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(self.find_best_transform(&capabilities, &config)?)
            .composite_alpha(self.find_best_composite_alpha(&capabilities, &config)?)
            .present_mode(self.find_best_present_mode(&config)?)
            .clipped(config.clipped);

        Ok(self.create_swapchain_direct(&mut info)?)
    }

    /// Creates a swapchain from a [`ash::vk::SwapchainCreateInfoKHR`].
    ///
    /// The surface and old_swapchain fields will be overwritten by this function. Any other fields
    /// or entries in the pNext chain will not be validated.
    pub fn create_swapchain_direct(&self, info: &mut vk::SwapchainCreateInfoKHR) -> VkResult<Arc<SurfaceSwapchain>> {
        let swapchain_khr = self.device.swapchain_khr().unwrap();

        info.surface = self.surface;

        let (mut guard, old_swapchain) = self.lock_current_swapchain();
        let swapchain_guard: Option<MutexGuard<vk::SwapchainKHR>> = if let Some(old_swapchain) = &old_swapchain {
            // We require that the info mutex is always locked before the swapchain mutex so this is safe
            let swapchain_guard = old_swapchain.swapchain.lock().unwrap();
            info.old_swapchain = *swapchain_guard;
            Some(swapchain_guard)
        } else {
            info.old_swapchain = vk::SwapchainKHR::null();
            None
        };

        let new_swapchain = unsafe {
            swapchain_khr.create_swapchain(info, None)
        }?;
        drop(swapchain_guard);

        let images = unsafe {
            swapchain_khr.get_swapchain_images(new_swapchain)
        }.map_err(|err| {
            unsafe {
                swapchain_khr.destroy_swapchain(new_swapchain, None);
            }
            guard.clear_current();
            err
        })?;

        let format = vk::SurfaceFormatKHR {
            format: info.image_format,
            color_space: info.image_color_space,
        };

        let new_swapchain = Arc::new(SurfaceSwapchain::new(self.weak.upgrade().unwrap(), new_swapchain, images.as_slice(), format, info.image_usage));
        guard.set_current(&new_swapchain);
        drop(guard);

        Ok(new_swapchain)
    }

    fn find_best_image_count(&self, capabilities: &vk::SurfaceCapabilitiesKHR, _: &SwapchainConfig) -> Result<u32, SwapchainCreateError> {
        if capabilities.max_image_count == 0 {
            Ok(std::cmp::max(capabilities.min_image_count, 3))

        } else {
            Ok(std::cmp::min(capabilities.max_image_count, std::cmp::max(capabilities.min_image_count, 3)))
        }
    }

    fn find_best_format(&self, config: &SwapchainConfig) -> Result<vk::SurfaceFormatKHR, SwapchainCreateError> {
        let supported = self.get_surface_formats()?;
        for format in config.formats.as_ref() {
            if supported.contains(format) {
                return Ok(*format);
            }
        }

        Err(SwapchainCreateError::Unsupported)
    }

    fn validate_extent(&self, capabilities: &vk::SurfaceCapabilitiesKHR, extent: Vec2u32) -> Result<vk::Extent2D, SwapchainCreateError> {
        if capabilities.max_image_extent.width == 0 || capabilities.max_image_extent.height == 0 {
            return Err(SwapchainCreateError::NoExtent)
        }

        if capabilities.max_image_extent.width < extent[0] ||
            capabilities.min_image_extent.width > extent[0] ||
            capabilities.max_image_extent.height < extent[1] ||
            capabilities.min_image_extent.height > extent[1] {
            return Err(SwapchainCreateError::Unsupported)
        }

        Ok(vk::Extent2D{ width: extent[0], height: extent[1] })
    }

    fn find_best_usage_flags(&self, capabilities: &vk::SurfaceCapabilitiesKHR, config: &SwapchainConfig) -> Result<vk::ImageUsageFlags, SwapchainCreateError> {
        if !capabilities.supported_usage_flags.contains(config.required_usage) {
            return Err(SwapchainCreateError::Unsupported);
        }

        let optional = capabilities.supported_usage_flags.bitand(config.optional_usage);
        Ok(config.required_usage.bitor(optional))
    }

    fn find_best_present_mode(&self, _: &SwapchainConfig) -> Result<vk::PresentModeKHR, SwapchainCreateError> {
        let supported = self.get_surface_present_modes()?;

        if supported.contains(&vk::PresentModeKHR::MAILBOX) {
            return Ok(vk::PresentModeKHR::MAILBOX);
        }

        Ok(vk::PresentModeKHR::FIFO)
    }

    fn find_best_transform(&self, capabilities: &vk::SurfaceCapabilitiesKHR, _: &SwapchainConfig) -> Result<vk::SurfaceTransformFlagsKHR, SwapchainCreateError> {
        if capabilities.supported_transforms.contains(capabilities.current_transform) {
            Ok(capabilities.current_transform)

        } else if capabilities.supported_transforms.contains(vk::SurfaceTransformFlagsKHR::IDENTITY) {
            Ok(vk::SurfaceTransformFlagsKHR::IDENTITY)

        } else if capabilities.supported_transforms.contains(vk::SurfaceTransformFlagsKHR::INHERIT) {
            Ok(vk::SurfaceTransformFlagsKHR::INHERIT)

        } else {
            let mut flag = 1u32;
            loop {
                let transform = vk::SurfaceTransformFlagsKHR::from_raw(Flags::from(flag));
                if capabilities.supported_transforms.contains(transform) {
                    return Ok(transform);
                }
                // The vulkan spec requires at least one bit to be set so we should panic if that's not the case
                flag = flag.checked_shr(1).unwrap();
            }
        }
    }

    fn find_best_composite_alpha(&self, capabilities: &vk::SurfaceCapabilitiesKHR, _: &SwapchainConfig) -> Result<vk::CompositeAlphaFlagsKHR, SwapchainCreateError> {
        if capabilities.supported_composite_alpha.contains(vk::CompositeAlphaFlagsKHR::OPAQUE) {
            Ok(vk::CompositeAlphaFlagsKHR::OPAQUE)

        } else if capabilities.supported_composite_alpha.contains(vk::CompositeAlphaFlagsKHR::INHERIT) {
            Ok(vk::CompositeAlphaFlagsKHR::INHERIT)

        } else {
            let mut flag = 1u32;
            loop {
                let comp = vk::CompositeAlphaFlagsKHR::from_raw(Flags::from(flag));
                if capabilities.supported_composite_alpha.contains(comp) {
                    return Ok(comp)
                }
                // The vulkan spec requires at least one bit to be set so we should panic if that's not the case
                flag = flag.checked_shr(1).unwrap();
            }
        }
    }

    /// Locks the current swapchain info. This function **must not** be called from inside the [`SurfaceSwapchain`]
    /// as it contains code to prevent a deadlock when the current [`SurfaceSwapchain`] is being dropped
    /// concurrently.
    fn lock_current_swapchain(&self) -> (MutexGuard<SurfaceSwapchainInfo>, Option<Arc<SurfaceSwapchain>>) {
        loop {
            let guard = self.current_swapchain.lock().unwrap();
            if let Ok(current) = guard.try_upgrade() {
                return (guard, current);
            }

            // The current swapchain is being dropped. We yield and wait for that to be completed
            drop(guard);
            std::thread::yield_now();
        }
    }
}

struct SurfaceSwapchainInfo {
    current_swapchain: Option<(UUID, Weak<SurfaceSwapchain>)>,
}

impl SurfaceSwapchainInfo {
    fn new() -> Self {
        Self {
            current_swapchain: None,
        }
    }

    fn try_upgrade(&self) -> Result<Option<Arc<SurfaceSwapchain>>, ()> {
        if let Some((_, weak)) = &self.current_swapchain {
            if let Some(arc) = weak.upgrade() {
                Ok(Some(arc))
            } else {
                Err(())
            }
        } else {
            Ok(None)
        }
    }

    fn is_current(&self, set_id: UUID) -> bool {
        if let Some((current, _)) = &self.current_swapchain {
            set_id == *current
        } else {
            false
        }
    }

    fn set_current(&mut self, swapchain: &Arc<SurfaceSwapchain>) {
        self.current_swapchain = Some((swapchain.set_id, Arc::downgrade(&swapchain)))
    }

    fn clear_current(&mut self) {
        self.current_swapchain = None;
    }
}

pub struct SwapchainConfig {
    pub formats: Box<[vk::SurfaceFormatKHR]>,
    pub required_usage: vk::ImageUsageFlags,
    pub optional_usage: vk::ImageUsageFlags,
    pub clipped: bool,
}

pub enum SwapchainCreateError {
    NoExtent,
    Unsupported,
    Vulkan(vk::Result),
}

impl From<vk::Result> for SwapchainCreateError {
    fn from(result: vk::Result) -> Self {
        Self::Vulkan(result)
    }
}

/// Wraps a swapchain of a [`DeviceSurface`]
///
/// The swpachain will be destroyed when this struct is dropped.
///
/// This struct implements [`ObjectSetProvider`] for access to swapchain images. The swapchain itself
/// can only be accessed by calling [`SurfaceSwapchain::get_swapchain`].
///
/// Holds an internal reference to the owning device surface keeping it alive.
pub struct SurfaceSwapchain {
    surface: Arc<DeviceSurface>,
    set_id: UUID,
    swapchain: Mutex<vk::SwapchainKHR>,
    images: Box<[(ImageId, vk::Image)]>,

    format: vk::SurfaceFormatKHR,
    usage: vk::ImageUsageFlags,
}

impl SurfaceSwapchain {
    fn new(surface: Arc<DeviceSurface>, swapchain: vk::SwapchainKHR, images: &[vk::Image], format: vk::SurfaceFormatKHR, usage: vk::ImageUsageFlags) -> Self {
        let mut image_data = Vec::with_capacity(images.len());
        for image in images {
            image_data.push((ImageId::new(), *image));
        }

        Self {
            surface,
            set_id: UUID::new(),
            swapchain: Mutex::new(swapchain),
            images: image_data.into_boxed_slice(),

            format,
            usage
        }
    }

    /// Returns the surface of this swapchain.
    pub fn get_surface(&self) -> &Arc<DeviceSurface> {
        &self.surface
    }

    /// Returns the handle of the swapchain.
    ///
    /// The since the swapchain must be externally synchronized a mutex is returned for the swapchain.
    pub fn get_swapchain(&self) -> &Mutex<vk::SwapchainKHR> {
        &self.swapchain
    }

    /// Returns all swpachain images and their ids.
    pub fn get_images(&self) -> &[(ImageId, vk::Image)] {
        self.images.as_ref()
    }

    /// Returns the format of the swapchain images
    pub fn get_image_format(&self) -> &vk::SurfaceFormatKHR {
        &self.format
    }

    /// Returns the usage flags of the swapchain images
    pub fn get_image_usage(&self) -> vk::ImageUsageFlags {
        self.usage
    }

    pub fn acquire_next_image(&self, timeout: u64, semaphore: Option<vk::Semaphore>, fence: Option<vk::Fence>) -> VkResult<(u32, bool)> {
        let swapchain_khr = self.surface.device.swapchain_khr().unwrap();
        let guard = self.swapchain.lock().unwrap();
        let result = unsafe {
            swapchain_khr.acquire_next_image(*guard, timeout, semaphore.unwrap_or(vk::Semaphore::null()), fence.unwrap_or(vk::Fence::null()))
        };
        drop(guard);

        result
    }
}

impl Debug for SurfaceSwapchain {
    fn fmt(&self, _: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Drop for SurfaceSwapchain {
    fn drop(&mut self) {
        let mut guard = self.surface.current_swapchain.lock().unwrap();

        // We do this inside the guard to propagate potential panics
        let swapchain_khr = self.surface.device.swapchain_khr().unwrap();
        let swapchain = self.swapchain.get_mut().unwrap();

        unsafe {
            swapchain_khr.destroy_swapchain(*swapchain, None)
        };

        if guard.is_current(self.set_id) {
            guard.clear_current();
        }
    }
}

impl ObjectSetProvider for SurfaceSwapchain {
    fn get_id(&self) -> UUID {
        self.set_id
    }

    fn get_handle(&self, id: UUID) -> Option<u64> {
        // We only expect a small number of images (<10) so a linear search will be the fastest option
        for (image_id, image) in self.images.as_ref() {
            if image_id.as_uuid() == id {
                return Some(image.as_raw());
            }
        }

        None
    }
}