use super::format::*;
use super::image::*;

use ash::vk;

#[derive(Copy, Clone)]
pub struct SwapchainImageSpec {
    pub format: &'static Format,
    pub color_space: vk::ColorSpaceKHR,
    pub extent: vk::Extent2D,
    pub array_layers: u32,
}

impl SwapchainImageSpec {
    pub const fn make(format: &'static Format, color_space: vk::ColorSpaceKHR, width: u32, height: u32) -> Self {
        Self {
            format,
            color_space,
            extent: vk::Extent2D {
                width,
                height
            },
            array_layers: 1
        }
    }

    pub const fn make_extent(format: &'static Format, color_space: vk::ColorSpaceKHR, extent: vk::Extent2D) -> Self {
        Self {
            format,
            color_space,
            extent,
            array_layers: 1
        }
    }

    pub const fn make_multiview(format: &'static Format, color_space: vk::ColorSpaceKHR, width: u32, height: u32, array_layers: u32) -> Self {
        Self {
            format,
            color_space,
            extent: vk::Extent2D {
                width,
                height
            },
            array_layers
        }
    }

    pub const fn make_multiview_extent(format: &'static Format, color_space: vk::ColorSpaceKHR, extent: vk::Extent2D, array_layers: u32) -> Self {
        Self {
            format,
            color_space,
            extent,
            array_layers
        }
    }

    pub const fn get_image_size(&self) -> ImageSize {
        ImageSize::make_2d_array(self.extent.width, self.extent.height, self.array_layers)
    }

    pub const fn as_image_spec(&self) -> ImageSpec {
        ImageSpec::new(self.get_image_size(), self.format, vk::SampleCountFlags::TYPE_1)
    }
}

#[derive(Copy, Clone)]
#[non_exhaustive]
pub struct SwapchainCreateDesc {
    pub min_image_count: u32,
    pub image_spec: SwapchainImageSpec,
    pub usage: vk::ImageUsageFlags,
    pub pre_transform: vk::SurfaceTransformFlagsKHR,
    pub composite_alpha: vk::CompositeAlphaFlagsKHR,
    pub present_mode: vk::PresentModeKHR,
    pub clipped: bool,
}

impl SwapchainCreateDesc {
    pub fn make(image_spec: SwapchainImageSpec, min_image_count: u32, usage: vk::ImageUsageFlags, present_mode: vk::PresentModeKHR) -> Self {
        SwapchainCreateDesc {
            min_image_count,
            image_spec,
            usage,
            pre_transform: vk::SurfaceTransformFlagsKHR::IDENTITY,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode,
            clipped: false,
        }
    }
}

pub struct SwapchainInstanceData {
    handle: vk::SwapchainKHR,
}

impl SwapchainInstanceData {
    pub fn new(handle: vk::SwapchainKHR) -> Self {
        Self {
            handle,
        }
    }

    pub unsafe fn get_handle(&self) -> vk::SwapchainKHR {
        self.handle
    }
}