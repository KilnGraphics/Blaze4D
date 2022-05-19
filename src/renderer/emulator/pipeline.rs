use std::cmp::Ordering;
use std::ffi::CStr;
use std::hash::{Hash, Hasher};
use std::mem::size_of;
use std::sync::{Arc, Mutex, Weak};
use std::sync::atomic::{AtomicU64, AtomicUsize};

use ash::vk;
use crate::device::device_utils::BlitPass;
use crate::objects::id::ImageId;
use crate::objects::{ObjectSet, ObjectSetProvider};

use crate::vk::DeviceEnvironment;

use crate::prelude::*;
use crate::vk::objects::allocator::{Allocation, AllocationStrategy};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct TestVertex {
    pub(crate) position: Vec3f32,
    pub(crate) uv: Vec2f32
}

const TEST_BUFFER_DATA: &[TestVertex] = &[
    TestVertex {
        position: Vec3f32::new(-0.5, 0.5, 0.1),
        uv: Vec2f32::new(0.0, 1.0)
    },
    TestVertex {
        position: Vec3f32::new(0.0, -0.5, 0.1),
        uv: Vec2f32::new(1.0, 1.0)
    },
    TestVertex {
        position: Vec3f32::new(0.5, 0.5, 0.1),
        uv: Vec2f32::new(1.0, 0.0)
    },
];

pub struct RenderPath {
    id: UUID,
    device: DeviceEnvironment,
    pipeline_layout: vk::PipelineLayout,
    render_pass: vk::RenderPass,
    pipeline: vk::Pipeline,

    test_buffer: vk::Buffer,
    test_allocation: Allocation,
}

impl RenderPath {
    pub(super) fn new(device: DeviceEnvironment) -> Self {
        let pipeline_layout = Self::create_pipeline_layout(&device);
        let render_pass = Self::create_render_pass(&device);

        let pipeline = Self::create_pipeline(&device, pipeline_layout, render_pass);

        let (test_buffer, test_allocation) = Self::create_test_buffer(&device);

        Self {
            id: UUID::new(),
            device,
            pipeline_layout,
            render_pass,
            pipeline,

            test_buffer,
            test_allocation
        }
    }

    fn create_pipeline_layout(device: &DeviceEnvironment) -> vk::PipelineLayout {
        let push_constants = vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::VERTEX,
            offset: 0,
            size: 2 * 16 * 4,
        };

        let info = vk::PipelineLayoutCreateInfo::builder()
            .push_constant_ranges(std::slice::from_ref(&push_constants));

        unsafe {
            device.vk().create_pipeline_layout(&info, None)
        }.unwrap()
    }

    fn create_render_pass(device: &DeviceEnvironment) -> vk::RenderPass {
        let attachments = [
            vk::AttachmentDescription::builder()
                .format(vk::Format::R8G8B8A8_SRGB)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .build(),
            vk::AttachmentDescription::builder()
                .format(vk::Format::D16_UNORM)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .build()
        ];

        let color_ref = vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let depth_ref = vk::AttachmentReference::builder()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(std::slice::from_ref(&color_ref))
            .depth_stencil_attachment(&depth_ref);

        let info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachments)
            .subpasses(std::slice::from_ref(&subpass));

        unsafe {
            device.vk().create_render_pass(&info, None)
        }.unwrap()
    }

    fn create_pipeline(device: &DeviceEnvironment, pipeline_layout: vk::PipelineLayout, render_pass: vk::RenderPass) -> vk::Pipeline {
        let (vertex_module, fragment_module) = Self::load_shaders(device);

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vertex_module)
                .name(CStr::from_bytes_with_nul(b"main\0").unwrap())
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(fragment_module)
                .name(CStr::from_bytes_with_nul(b"main\0").unwrap())
                .build()
        ];

        let binding = vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride((size_of::<Vec3f32>() + size_of::<Vec2f32>()) as u32)
            .input_rate(vk::VertexInputRate::VERTEX);

        let input_descriptions = [
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: 0
            },
            vk::VertexInputAttributeDescription {
                location: 1,
                binding: 0,
                format: vk::Format::R32G32_SFLOAT,
                offset: size_of::<Vec3f32>() as u32,
            }
        ];

        let input_state = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(std::slice::from_ref(&binding))
            .vertex_attribute_descriptions(&input_descriptions);

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .scissor_count(1);

        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false)
            .line_width(1f32);

        let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(false);

        let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::GREATER)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false);

        let attachment = vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(false)
            .color_write_mask(vk::ColorComponentFlags::RGBA);

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .attachments(std::slice::from_ref(&attachment));

        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];

        let dynamic_state = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&dynamic_states);

        let info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .depth_stencil_state(&depth_stencil_state)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0);

        let pipeline = *unsafe {
            device.vk().create_graphics_pipelines(vk::PipelineCache::null(), std::slice::from_ref(&info), None)
        }.unwrap().get(0).unwrap();

        unsafe {
            device.vk().destroy_shader_module(vertex_module, None);
            device.vk().destroy_shader_module(fragment_module, None);
        }

        pipeline
    }

    fn load_shaders(device: &DeviceEnvironment) -> (vk::ShaderModule, vk::ShaderModule) {
        let info = vk::ShaderModuleCreateInfo::builder()
            .code(crate::util::slice::from_byte_slice(BASIC_VERTEX_SHADER));

        let vertex = unsafe {
            device.vk().create_shader_module(&info, None)
        }.unwrap();

        let info = vk::ShaderModuleCreateInfo::builder()
            .code(crate::util::slice::from_byte_slice(BASIC_FRAGMENT_SHADER));

        let fragment = unsafe {
            device.vk().create_shader_module(&info, None)
        }.unwrap();

        (vertex, fragment)
    }

    fn create_test_buffer(device: &DeviceEnvironment) -> (vk::Buffer, Allocation) {
        let data = crate::util::slice::to_byte_slice(TEST_BUFFER_DATA);

        let info = vk::BufferCreateInfo::builder()
            .size(data.len() as u64)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe {
            device.vk().create_buffer(&info, None)
        }.unwrap();

        let alloc = device.get_allocator().allocate_buffer_memory(buffer, &AllocationStrategy::AutoGpuCpu).unwrap();

        unsafe {
            device.vk().bind_buffer_memory(buffer, alloc.memory(), alloc.offset())
        }.unwrap();

        let dst = unsafe {
            std::slice::from_raw_parts_mut(alloc.mapped_ptr().unwrap().as_ptr() as *mut u8, data.len())
        };
        dst.copy_from_slice(data);

        (buffer, alloc)
    }
}

impl Drop for RenderPath {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().destroy_pipeline(self.pipeline, None);
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

    pub(super) fn get_tmp_vertex_size(&self) -> usize {
        size_of::<TestVertex>()
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

    pub(super) fn begin_render_pass(&self, command_buffer: vk::CommandBuffer, index: usize) {
        let device = &self.render_path.device;

        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [1.0, 0.0, 0.0, 1.0],
                }
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 0.0,
                    stencil: 0
                }
            }
        ];

        let info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_path.render_pass)
            .framebuffer(self.render_objects[index].framebuffer)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D{ x: 0, y: 0 },
                extent: vk::Extent2D{ width: self.render_size[0], height: self.render_size[1] }
            })
            .clear_values(&clear_values);

        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: self.render_size[0] as f32,
            height: self.render_size[1] as f32,
            min_depth: 0.0,
            max_depth: 1.0
        };

        let scissor = vk::Rect2D {
            offset: vk::Offset2D{ x: 0, y: 0 },
            extent: vk::Extent2D{ width: self.render_size[0], height: self.render_size[1] }
        };

        unsafe {
            device.vk().cmd_begin_render_pass(command_buffer, &info, vk::SubpassContents::INLINE);
            device.vk().cmd_set_viewport(command_buffer, 0, std::slice::from_ref(&viewport));
            device.vk().cmd_set_scissor(command_buffer, 0, std::slice::from_ref(&scissor));
            device.vk().cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.render_path.pipeline);
        };
    }

    pub(super) fn set_world_ndc_mat(&self, command_buffer: vk::CommandBuffer, mat: Mat4f32) {
        unsafe {
            self.render_path.device.vk().cmd_push_constants(
                command_buffer,
                self.render_path.pipeline_layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                crate::util::slice::to_byte_slice(std::slice::from_ref(&mat))
            )
        }
    }

    pub(super) fn set_model_world_mat(&self, command_buffer: vk::CommandBuffer, mat: Mat4f32) {
        unsafe {
            self.render_path.device.vk().cmd_push_constants(
                command_buffer,
                self.render_path.pipeline_layout,
                vk::ShaderStageFlags::VERTEX,
                4 * 16,
                crate::util::slice::to_byte_slice(std::slice::from_ref(&mat))
            )
        }
    }

    pub(super) fn test_draw(&self, command_buffer: vk::CommandBuffer) {
        let offsets = [0];
        unsafe {
            self.render_path.device.vk().cmd_bind_vertex_buffers(command_buffer, 0, std::slice::from_ref(&self.render_path.test_buffer), &offsets);
            self.render_path.device.vk().cmd_draw(command_buffer, 3, 1, 0, 0);
        }
    }

    pub(super) fn prepare_index(&self, index: usize) -> (vk::Semaphore, u64) {
        let instance = &self.render_objects[index];
        let current = instance.ready_value.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let wait_info = vk::SemaphoreWaitInfo::builder()
            .semaphores(std::slice::from_ref(&instance.ready_semaphore))
            .values(std::slice::from_ref(&current));

        unsafe {
            self.render_path.device.vk().wait_semaphores(&wait_info, u64::MAX)
        }.unwrap();

        (instance.ready_semaphore, current + 1)
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

    pub(super) fn record(&self, command_buffer: vk::CommandBuffer, src_index: usize, dst_index: usize) {
        self.blit.record_blit(command_buffer, self.descriptors[src_index], self.dst_objects[dst_index].1, self.output_size, None);
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

        let views: Box<_> = config.render_objects.iter().map(|frame| frame.color_view).collect();

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