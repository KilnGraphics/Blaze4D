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
use crate::device::device_utils::BlitPass;
use crate::renderer::emulator::buffer::{BufferAllocation, BufferPool, BufferSubAllocator};
use crate::renderer::emulator::frame::FrameManager;
use crate::renderer::emulator::pipeline::{Pipeline, PipelineId, PipelineManager};
use crate::renderer::emulator::render_worker::{DrawTask, Share};
use crate::device::transfer::{BufferAvailabilityOp, BufferTransferRanges, Transfer};
use crate::objects::id::ImageId;
use crate::objects::{ObjectSet, ObjectSetProvider};
use crate::vk::objects::buffer::Buffer;
use crate::vk::objects::semaphore::SemaphoreOps;

use crate::prelude::*;
use crate::vk::DeviceEnvironment;
use crate::vk::objects::allocator::{Allocation, AllocationStrategy};

pub(crate) struct EmulatorRenderer {
    device: DeviceEnvironment,
    worker: Arc<Share>,
    frame_manager: FrameManager,
    buffer_pool: Mutex<BufferPool>,
    pipelines: PipelineManager,
    current_config: Mutex<Option<Arc<StableObjects>>>,
}

impl EmulatorRenderer {
    pub(crate) fn new(device: DeviceEnvironment) -> Arc<Self> {
        Arc::new(Self {
            device: device.clone(),
            worker: Arc::new(Share::new(device.clone())),
            frame_manager: FrameManager::new(),
            buffer_pool: Mutex::new(BufferPool::new(device.clone())),
            pipelines: PipelineManager::new(device),
            current_config: Mutex::new(None),
        })
    }

    pub fn configure_framebuffer(&self, render_size: Vec2u32, output_size: Vec2u32, output_images: &[ImageId], output_format: vk::Format, image_set: ObjectSet, post_layout: vk::ImageLayout) {
        let objects = StableObjects::new(self.device.clone(), 3, render_size, self.pipelines.get_render_pass(), output_size, output_images, output_format, post_layout, image_set);

        let mut guard = self.current_config.lock().unwrap();
        *guard = Some(Arc::new(objects));
    }

    pub fn reset_framebuffer(&self) {
        let mut guard = self.current_config.lock().unwrap();
        *guard = None;
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
    descriptor_pool: vk::DescriptorPool,
    frame_objects: Box<[StableFrameObjects]>,
    render_size: Vec2u32,
    output_blit: BlitPass,
    output_size: Vec2u32,
    output_views: Box<[vk::ImageView]>,
    output_framebuffers: Box<[vk::Framebuffer]>,
    output_image_set: ObjectSet,
}

impl StableObjects {
    fn new(device: DeviceEnvironment, frame_count: usize, render_size: Vec2u32, render_pass: vk::RenderPass, output_size: Vec2u32, output_images: &[ImageId], output_format: vk::Format, output_layout: vk::ImageLayout, image_set: ObjectSet) -> Self {
        let blit_pass = device.get_utils().blit_utils().create_blit_pass(output_format, vk::AttachmentLoadOp::DONT_CARE, vk::ImageLayout::UNDEFINED, output_layout);
        let descriptor_pool = Self::create_descriptor_pool(&device, frame_count);

        let frame_objects = repeat_with(|| StableFrameObjects::new(&device, render_size, render_pass, &blit_pass, descriptor_pool)).take(frame_count).collect();

        let output_views = Self::create_output_image_views(&device, output_images, output_format, &image_set).into_boxed_slice();
        let output_framebuffers = Self::create_output_framebuffers(&blit_pass, output_views.as_ref(), output_size).into_boxed_slice();

        Self {
            device,
            descriptor_pool,
            frame_objects,
            render_size,
            output_blit: blit_pass,
            output_size,
            output_views,
            output_framebuffers,
            output_image_set: image_set,
        }
    }

    fn create_descriptor_pool(device: &DeviceEnvironment, count: usize) -> vk::DescriptorPool {
        let size = vk::DescriptorPoolSize::builder()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(count as u32);

        let info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(count as u32)
            .pool_sizes(std::slice::from_ref(&size));

        unsafe {
            device.vk().create_descriptor_pool(&info, None)
        }.unwrap()
    }

    fn create_output_image_views(device: &DeviceEnvironment, images: &[ImageId], format: vk::Format, set: &ObjectSet) -> Vec<vk::ImageView> {
        let mut views = Vec::with_capacity(images.len());
        for image in images {
            let image = set.get(*image).unwrap();

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
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1
                });

            let view = unsafe {
                device.vk().create_image_view(&info, None)
            }.unwrap();

            views.push(view);
        }

        views
    }

    fn create_output_framebuffers(blit: &BlitPass, views: &[vk::ImageView], size: Vec2u32) -> Vec<vk::Framebuffer> {
        let mut framebuffers = Vec::with_capacity(views.len());
        for view in views {
            framebuffers.push(blit.create_framebuffer(*view, size).unwrap());
        }

        framebuffers
    }
}

struct StableFrameObjects {
    color_image: vk::Image,
    color_view: vk::ImageView,
    depth_stencil_image: vk::Image,
    depth_stencil_view: vk::ImageView,
    framebuffer: vk::Framebuffer,
    blit_descriptor_set: vk::DescriptorSet,
    wait_fence: vk::Fence,
    color_alloc: Allocation,
    depth_stencil_alloc: Allocation,
}

impl StableFrameObjects {
    fn new(device: &DeviceEnvironment, size: Vec2u32, render_pass: vk::RenderPass, blit_pass: &BlitPass, descriptor_pool: vk::DescriptorPool) -> Self {
        let (color_image, color_alloc) =
            Self::create_image(device, size, vk::Format::R8G8B8A8_SRGB, vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED);
        let (depth_stencil_image, depth_stencil_alloc) =
            Self::create_image(device, size, vk::Format::D16_UNORM, vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::INPUT_ATTACHMENT | vk::ImageUsageFlags::SAMPLED);

        let color_view = Self::create_view(device, color_image, vk::Format::R8G8B8A8_SRGB, vk::ImageAspectFlags::COLOR);
        let depth_stencil_view = Self::create_view(device, depth_stencil_image, vk::Format::D16_UNORM, vk::ImageAspectFlags::DEPTH);

        let framebuffer = Self::create_framebuffer(device, color_view, depth_stencil_view, render_pass, size);

        let blit_descriptor_set = *blit_pass.create_descriptor_sets(descriptor_pool, std::slice::from_ref(&depth_stencil_view)).unwrap().get(0).unwrap();

        let wait_fence = Self::create_fence(device);

        Self {
            color_image,
            color_view,
            depth_stencil_image,
            depth_stencil_view,
            framebuffer,
            blit_descriptor_set,
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