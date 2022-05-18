mod pipeline;
mod buffer;
mod frame;
mod render_worker;

use std::iter::repeat_with;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::sync::atomic::{AtomicU64, Ordering};

use ash::vk;

use concurrent_queue::ConcurrentQueue;

use crate::device::device_utils::BlitPass;
use crate::renderer::emulator::buffer::{BufferAllocation, BufferPool, BufferSubAllocator};
use crate::renderer::emulator::frame::{Frame, FrameId};
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
    weak: Weak<EmulatorRenderer>,
    device: DeviceEnvironment,
    worker: Arc<Share>,
    next_frame_id: AtomicU64,
    buffer_pool: Mutex<BufferPool>,
    pipelines: PipelineManager,
}

impl EmulatorRenderer {
    pub(crate) fn new(device: DeviceEnvironment) -> Arc<Self> {
        Arc::new_cyclic(|weak| {
            Self {
                weak: weak.clone(),
                device: device.clone(),
                worker: Arc::new(Share::new(device.clone())),
                next_frame_id: AtomicU64::new(1),
                buffer_pool: Mutex::new(BufferPool::new(device.clone())),
                pipelines: PipelineManager::new(device),
            }
        })
    }

    pub fn create_render_configuration(&self, render_size: Vec2u32) -> Arc<RenderConfiguration> {
        Arc::new(RenderConfiguration::new(
            self.weak.upgrade().unwrap(),
            self.device.clone(),
            render_size,
            3
        ))
    }

    pub fn create_output_configuration(
        &self,
        render_configuration: Arc<RenderConfiguration>,
        output_size: Vec2u32,
        dst_images: &[ImageId],
        dst_set: ObjectSet,
        dst_format: vk::Format,
        final_layout: vk::ImageLayout
    ) -> Arc<OutputConfiguration> {

        Arc::new(OutputConfiguration::new(
            render_configuration,
            output_size,
            dst_images,
            dst_set,
            dst_format,
            final_layout
        ))
    }

    pub fn start_frame(&self, configuration: Arc<RenderConfiguration>) -> Frame {
        let id = FrameId::from_raw(self.next_frame_id.fetch_add(1, Ordering::SeqCst));
        Frame::new(id, self.weak.upgrade().unwrap(), configuration)
    }
}

pub struct RenderConfiguration {
    renderer: Arc<EmulatorRenderer>,
    device: DeviceEnvironment,
    frame_objects: Box<[RenderFrameObjects]>,
    render_size: Vec2u32,
}

impl RenderConfiguration {
    fn new(renderer: Arc<EmulatorRenderer>, device: DeviceEnvironment, render_size: Vec2u32, frame_count: usize) -> Self {
        let render_pass = renderer.pipelines.get_render_pass();
        let frame_objects: Box<_> = repeat_with(|| RenderFrameObjects::new(&device, render_size, render_pass)).take(frame_count).collect();

        Self {
            renderer,
            device,
            frame_objects,
            render_size,
        }
    }
}

impl Drop for RenderConfiguration {
    fn drop(&mut self) {
        for frame in self.frame_objects.iter_mut() {
            frame.destroy(&self.device);
        }
    }
}

struct RenderFrameObjects {
    color_image: vk::Image,
    depth_stencil_image: vk::Image,
    color_view: vk::ImageView,
    depth_stencil_view: vk::ImageView,
    framebuffer: vk::Framebuffer,
    color_allocation: Option<Allocation>,
    depth_stencil_allocation: Option<Allocation>,
}

impl RenderFrameObjects {
    fn new(device: &DeviceEnvironment, size: Vec2u32, render_pass: vk::RenderPass) -> Self {
        let (color_image, color_allocation) = Self::create_image(
            device,
            size,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED
        );

        let (depth_stencil_image, depth_stencil_allocation) = Self::create_image(
            device,
            size,
            vk::Format::D16_UNORM,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::SAMPLED
        );

        let color_view = Self::create_view(device, color_image, vk::Format::R8G8B8A8_SRGB, vk::ImageAspectFlags::COLOR);
        let depth_stencil_view = Self::create_view(device, depth_stencil_image, vk::Format::D16_UNORM, vk::ImageAspectFlags::DEPTH);

        let framebuffer = Self::create_framebuffer(device, color_view, depth_stencil_view, render_pass, size);

        Self {
            color_image,
            depth_stencil_image,
            color_view,
            depth_stencil_view,
            framebuffer,
            color_allocation: Some(color_allocation),
            depth_stencil_allocation: Some(depth_stencil_allocation),
        }
    }

    fn destroy(&mut self, device: &DeviceEnvironment) {
        unsafe {
            device.vk().destroy_framebuffer(self.framebuffer, None);
            device.vk().destroy_image_view(self.depth_stencil_view, None);
            device.vk().destroy_image_view(self.color_view, None);
            device.vk().destroy_image(self.depth_stencil_image, None);
            device.vk().destroy_image(self.color_image, None);
        }
        device.get_allocator().free(self.depth_stencil_allocation.take().unwrap());
        device.get_allocator().free(self.color_allocation.take().unwrap());
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
}

pub struct OutputConfiguration {
    render_configuration: Arc<RenderConfiguration>,
    descriptor_pool: vk::DescriptorPool,
    descriptors: Box<[vk::DescriptorSet]>,
    output_size: Vec2u32,
    blit: BlitPass,
    dst_objects: Box<[(vk::ImageView, vk::Framebuffer)]>,
    dst_set: ObjectSet,
}

impl OutputConfiguration {
    fn new(render_configuration: Arc<RenderConfiguration>, output_size: Vec2u32, dst_images: &[ImageId], dst_set: ObjectSet, dst_format: vk::Format, final_layout: vk::ImageLayout) -> Self {
        let blit = render_configuration.device.get_utils().blit_utils().create_blit_pass(dst_format, vk::AttachmentLoadOp::DONT_CARE, vk::ImageLayout::UNDEFINED, final_layout);

        let (descriptor_pool, descriptors) = Self::create_descriptors(&render_configuration, &blit);

        let dst_objects = Self::create_dst_objects(&render_configuration.device, &blit, dst_images, &dst_set, dst_format, output_size);

        Self {
            render_configuration,
            descriptor_pool,
            descriptors,
            output_size,
            blit,
            dst_objects,
            dst_set,
        }
    }

    fn create_descriptors(config: &RenderConfiguration, blit: &BlitPass) -> (vk::DescriptorPool, Box<[vk::DescriptorSet]>) {
        let count = config.frame_objects.len() as u32;
        let size = vk::DescriptorPoolSize::builder()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(count);

        let info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(count)
            .pool_sizes(std::slice::from_ref(&size));

        let descriptor_pool = unsafe {
            config.device.vk().create_descriptor_pool(&info, None)
        }.unwrap();

        let views: Box<_> = config.frame_objects.iter().map(|frame| frame.depth_stencil_view).collect();

        let descriptors = blit.create_descriptor_sets(descriptor_pool, &views).unwrap();

        (descriptor_pool, descriptors.into_boxed_slice())
    }

    fn create_dst_objects(device: &DeviceEnvironment, blit: &BlitPass, dst_images: &[ImageId], dst_set: &ObjectSet, dst_format: vk::Format, size: Vec2u32) -> Box<[(vk::ImageView, vk::Framebuffer)]> {
        dst_images.iter().map(|id| {
            let image = dst_set.get(*id).unwrap();

            let info = vk::ImageViewCreateInfo::builder()
                .image(image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(dst_format)
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

            let framebuffer = blit.create_framebuffer(view, size).unwrap();

            (view, framebuffer)
        }).collect()
    }
}

impl Drop for OutputConfiguration {
    fn drop(&mut self) {
        let device = &self.render_configuration.device;
        unsafe {
            for (image_view, framebuffer) in self.dst_objects.iter() {
                device.vk().destroy_framebuffer(*framebuffer, None);
                device.vk().destroy_image_view(*image_view, None);
            }
            device.vk().destroy_descriptor_pool(self.descriptor_pool, None);
        }
    }
}