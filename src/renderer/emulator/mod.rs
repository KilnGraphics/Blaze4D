mod pipeline;
mod buffer;
mod frame;
mod render_worker;

use std::iter::repeat_with;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::atomic::{AtomicU64, Ordering};
use ash::vk;
use concurrent_queue::ConcurrentQueue;
use crate::renderer::emulator::buffer::{BufferAllocation, BufferPool, BufferSubAllocator};
use crate::renderer::emulator::frame::FrameManager;
use crate::renderer::emulator::pipeline::{Pipeline, PipelineId, PipelineManager};
use crate::renderer::emulator::render_worker::{DrawTask, Share};
use crate::renderer::swapchain_manager::SwapchainInstance;
use crate::device::transfer::{BufferAvailabilityOp, BufferTransferRanges, Transfer};
use crate::vk::objects::buffer::Buffer;
use crate::vk::objects::semaphore::SemaphoreOps;

use crate::prelude::*;
use crate::vk::DeviceEnvironment;
use crate::vk::objects::allocator::{Allocation, AllocationStrategy};

struct EmulatorRendererShare {
    transfer: Arc<Transfer>,
    worker: Arc<Share>,
    frame_manager: FrameManager,
    buffer_pool: Mutex<BufferPool>,
    pipelines: PipelineManager,
}

impl EmulatorRendererShare {
    fn new(device: DeviceEnvironment, transfer: Arc<Transfer>) -> Self {
        Self {
            transfer,
            worker: Arc::new(Share::new(device.clone())),
            frame_manager: FrameManager::new(),
            buffer_pool: Mutex::new(BufferPool::new(device.clone())),
            pipelines: PipelineManager::new(device)
        }
    }
}

pub struct EmulatorRenderer(Arc<EmulatorRendererShare>);

impl EmulatorRenderer {
    pub fn new(device: DeviceEnvironment, transfer: Arc<Transfer>) -> Self {
        Self(Arc::new(EmulatorRendererShare::new(device, transfer)))
    }

    fn register_pipeline(&self) -> PipelineId {
        todo!()
    }

    pub fn start_frame(&self) {
        todo!()
    }
}

struct StableObjects {
    device: DeviceEnvironment,
    surface_size: Vec2u32,
    frame_objects: Box<[StableFrameObjects]>,
    output_size: Vec2u32,
    output_framebuffer: vk::Framebuffer,
}

impl StableObjects {
    fn new(device: DeviceEnvironment, frame_count: usize, size: Vec2u32, render_pass: vk::RenderPass) -> Self {
        let frame_objects = repeat_with(|| StableFrameObjects::new(&device, size, render_pass)).take(frame_count).collect();

        Self {
            device,
            surface_size: size,
            frame_objects,
            output_size: size,
            output_framebuffer: vk::Framebuffer::null()
        }
    }

    fn create_output_framebuffer(device: &DeviceEnvironment, size: Vec2u32, format: vk::Format, usage: vk::ImageUsageFlags) -> vk::Framebuffer {
        let view_formats = [vk::Format::R8G8B8A8_SRGB, format];
        let view_formats = if format == vk::Format::R8G8B8A8_SRGB {
            &view_formats[0..1]
        } else {
            &view_formats
        };

        let mut image_info = vk::FramebufferAttachmentImageInfo::builder()
            .usage(usage)
            .width(size[0])
            .height(size[1])
            .layer_count(1)
            .view_formats(view_formats);

        let info = vk::FramebufferCreateInfo::builder()
            .flags(vk::FramebufferCreateFlags::IMAGELESS);

        todo!()
    }
}

struct StableFrameObjects {
    color_image: vk::Image,
    color_view: vk::ImageView,
    depth_stencil_image: vk::Image,
    depth_stencil_view: vk::ImageView,
    framebuffer: vk::Framebuffer,
    wait_fence: vk::Fence,
    color_alloc: Allocation,
    depth_stencil_alloc: Allocation,
}

impl StableFrameObjects {
    fn new(device: &DeviceEnvironment, size: Vec2u32, render_pass: vk::RenderPass) -> Self {
        let (color_image, color_alloc) =
            Self::create_image(device, size, vk::Format::R8G8B8A8_SRGB, vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC);
        let (depth_stencil_image, depth_stencil_alloc) =
            Self::create_image(device, size, vk::Format::D16_UNORM, vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::INPUT_ATTACHMENT);

        let color_view = Self::create_view(device, color_image, vk::Format::R8G8B8A8_SRGB, vk::ImageAspectFlags::COLOR);
        let depth_stencil_view = Self::create_view(device, depth_stencil_image, vk::Format::D16_UNORM, vk::ImageAspectFlags::DEPTH);

        let framebuffer = Self::create_framebuffer(device, color_view, depth_stencil_view, render_pass, size);

        let wait_fence = Self::create_fence(device);

        Self {
            color_image,
            color_view,
            depth_stencil_image,
            depth_stencil_view,
            framebuffer,
            wait_fence,
            color_alloc,
            depth_stencil_alloc
        }
    }

    fn destroy(&self, device: &DeviceEnvironment) {
        todo!()
    }

    pub fn get_framebuffer(&self) -> vk::Framebuffer {
        self.framebuffer
    }

    pub fn get_wait_fence(&self) -> vk::Fence {
        self.wait_fence
    }

    fn create_image(device: &DeviceEnvironment, size: Vec2u32, format: vk::Format, usage: vk::ImageUsageFlags) -> (vk::Image, Allocation) {
        let info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(format)
            .extent(vk::Extent3D {
                width: size[0],
                height: size[1],
                depth: 1
            })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED);

        let image = unsafe {
            device.vk().create_image(&info, None)
        }.unwrap();

        let alloc = device.get_allocator().allocate_image_memory(image, &AllocationStrategy::AutoGpuOnly).unwrap();

        unsafe {
            device.vk().bind_image_memory(image, alloc.memory(), alloc.offset())
        }.unwrap();

        (image, alloc)
    }

    fn create_view(device: &DeviceEnvironment, image: vk::Image, format: vk::Format, aspect_mask: vk::ImageAspectFlags) -> vk::ImageView {
        let info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY
            })
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask,
                base_mip_level: 0,
                level_count: vk::REMAINING_MIP_LEVELS,
                base_array_layer: 0,
                layer_count: vk::REMAINING_ARRAY_LAYERS
            });

        unsafe {
            device.vk().create_image_view(&info, None)
        }.unwrap()
    }

    fn create_framebuffer(device: &DeviceEnvironment, color_view: vk::ImageView, depth_stencil_view: vk::ImageView, render_pass: vk::RenderPass, size: Vec2u32) -> vk::Framebuffer {
        let attachments = [color_view, depth_stencil_view];

        let info = vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(&attachments)
            .width(size[0])
            .height(size[1])
            .layers(1);

        unsafe {
            device.vk().create_framebuffer(&info, None)
        }.unwrap()
    }

    fn create_fence(device: &DeviceEnvironment) -> vk::Fence {
        let info = vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED);

        unsafe {
            device.vk().create_fence(&info, None)
        }.unwrap()
    }
}