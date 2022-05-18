use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, Weak};
use std::sync::atomic::{AtomicU64, AtomicUsize};

use ash::vk;
use crate::device::device_utils::BlitPass;
use crate::objects::id::ImageId;
use crate::objects::{ObjectSet, ObjectSetProvider};

use crate::vk::DeviceEnvironment;

use crate::prelude::*;
use crate::vk::objects::allocator::{Allocation, AllocationStrategy};

pub struct RenderPath {
    id: UUID,
    device: DeviceEnvironment,
    pipeline_layout: vk::PipelineLayout,
    render_pass: vk::RenderPass,
}

impl RenderPath {
    pub(super) fn new(device: DeviceEnvironment) -> Self {
        let pipeline_layout = Self::create_pipeline_layout(&device);
        let render_pass = Self::create_render_pass(&device);

        Self {
            id: UUID::new(),
            device,
            pipeline_layout,
            render_pass,
        }
    }

    fn create_pipeline_layout(device: &DeviceEnvironment) -> vk::PipelineLayout {
        let info = vk::PipelineLayoutCreateInfo::builder();

        unsafe {
            device.vk().create_pipeline_layout(&info, None)
        }.unwrap()
    }

    fn create_render_pass(device: &DeviceEnvironment) -> vk::RenderPass {
        let attachment = vk::AttachmentDescription::builder()
            .format(vk::Format::R8G8B8A8_SRGB)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);

        let attachment_ref = vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(std::slice::from_ref(&attachment_ref));

        let info = vk::RenderPassCreateInfo::builder()
            .attachments(std::slice::from_ref(&attachment))
            .subpasses(std::slice::from_ref(&subpass));

        unsafe {
            device.vk().create_render_pass(&info, None)
        }.unwrap()
    }
}

impl Drop for RenderPath {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().destroy_render_pass(self.render_pass, None);
            self.device.vk().destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }
}

impl PartialEq for RenderPath {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl Eq for RenderPath {
}

impl PartialOrd for RenderPath {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for RenderPath {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl Hash for RenderPath {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

pub struct RenderConfiguration {
    render_path: Arc<RenderPath>,
    next_index: AtomicUsize,
    render_objects: Box<[RenderObjects]>,
    render_size: Vec2u32,
}

impl RenderConfiguration {
    pub fn new(render_path: Arc<RenderPath>, render_size: Vec2u32, max_concurrent: usize) -> Self {
        let device = &render_path.device;
        let frame_objects: Box<_> = std::iter::repeat_with(|| RenderObjects::new(device, render_size, render_path.render_pass)).take(max_concurrent).collect();

        Self {
            render_path,
            next_index: AtomicUsize::new(0),
            render_objects: frame_objects,
            render_size,
        }
    }

    pub(super) fn get_next_index(&self) -> usize {
        loop {
            let old = self.next_index.load(std::sync::atomic::Ordering::SeqCst);
            let new = (old + 1) % self.render_objects.len();
            if self.next_index.compare_exchange(old, new, std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst).is_ok() {
                return new;
            }
        }
    }
}

impl Drop for RenderConfiguration {
    fn drop(&mut self) {
        let device = &self.render_path.device;
        for frame in self.render_objects.iter_mut() {
            frame.destroy(device);
        }
    }
}

struct RenderObjects {
    ready_semaphore: vk::Semaphore,
    ready_value: AtomicU64,
    color_image: vk::Image,
    depth_stencil_image: vk::Image,
    color_view: vk::ImageView,
    depth_stencil_view: vk::ImageView,
    framebuffer: vk::Framebuffer,
    color_allocation: Option<Allocation>,
    depth_stencil_allocation: Option<Allocation>,
}

impl RenderObjects {
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

        let ready_semaphore = Self::create_semaphore(device);

        Self {
            ready_semaphore,
            ready_value: AtomicU64::new(0),
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
            device.vk().destroy_semaphore(self.ready_semaphore, None);
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

    fn create_framebuffer(device: &DeviceEnvironment, color_view: vk::ImageView, _: vk::ImageView, render_pass: vk::RenderPass, size: Vec2u32) -> vk::Framebuffer {
        let attachments = [color_view];

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

    fn create_semaphore(device: &DeviceEnvironment) -> vk::Semaphore {
        let mut type_info = vk::SemaphoreTypeCreateInfo::builder()
            .semaphore_type(vk::SemaphoreType::TIMELINE)
            .initial_value(0);

        let info = vk::SemaphoreCreateInfo::builder()
            .push_next(&mut type_info);

        unsafe {
            device.vk().create_semaphore(&info, None)
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
    pub fn new(
        render_configuration: Arc<RenderConfiguration>,
        output_size: Vec2u32,
        dst_images: &[ImageId],
        dst_set: ObjectSet,
        dst_format: vk::Format,
        final_layout: vk::ImageLayout
    ) -> Self {
        let device = &render_configuration.render_path.device;
        let blit = device.get_utils().blit_utils().create_blit_pass(dst_format, vk::AttachmentLoadOp::DONT_CARE, vk::ImageLayout::UNDEFINED, final_layout);

        let (descriptor_pool, descriptors) = Self::create_descriptors(&render_configuration, &blit);

        let dst_objects = Self::create_dst_objects(device, &blit, dst_images, &dst_set, dst_format, output_size);

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
        let device = &config.render_path.device;

        let count = config.render_objects.len() as u32;
        let size = vk::DescriptorPoolSize::builder()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(count);

        let info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(count)
            .pool_sizes(std::slice::from_ref(&size));

        let descriptor_pool = unsafe {
            device.vk().create_descriptor_pool(&info, None)
        }.unwrap();

        let views: Box<_> = config.render_objects.iter().map(|frame| frame.depth_stencil_view).collect();

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
        let device = &self.render_configuration.render_path.device;
        unsafe {
            for (image_view, framebuffer) in self.dst_objects.iter() {
                device.vk().destroy_framebuffer(*framebuffer, None);
                device.vk().destroy_image_view(*image_view, None);
            }
            device.vk().destroy_descriptor_pool(self.descriptor_pool, None);
        }
    }
}

const BASIC_VERTEX_SHADER: &'static [u8] = include_bytes!(concat!(env!("B4D_RESOURCE_DIR"), "debug/basic_vert.spv"));
const BASIC_FRAGMENT_SHADER: &'static [u8] = include_bytes!(concat!(env!("B4D_RESOURCE_DIR"), "debug/basic_frag.spv"));