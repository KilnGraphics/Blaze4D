use std::ffi::CStr;
use std::iter::repeat;
use std::sync::{Arc, Weak};

use ash::prelude::VkResult;
use ash::vk;
use bytemuck::cast_slice;
use include_bytes_aligned::include_bytes_aligned;
use crate::allocator::Allocator;

use crate::prelude::*;

pub fn create_shader_from_bytes(device: &DeviceFunctions, code: &[u8]) -> VkResult<vk::ShaderModule> {
    let info = vk::ShaderModuleCreateInfo::builder()
        .code(cast_slice(code));

    unsafe {
        device.vk.create_shader_module(&info, None)
    }
}

pub struct DeviceUtils {
    blit_utils: BlitUtils,
}

impl DeviceUtils {
    pub fn new(device: Arc<DeviceFunctions>, _: Arc<Allocator>) -> Arc<Self> {
        Arc::new_cyclic(|weak| {
            Self {
                blit_utils: BlitUtils::new(weak.clone(), device)
            }
        })
    }

    pub fn blit_utils(&self) -> &BlitUtils {
        &self.blit_utils
    }
}

pub struct BlitUtils {
    utils: Weak<DeviceUtils>,
    device: Arc<DeviceFunctions>,
    vertex_shader: vk::ShaderModule,
    fragment_shader: vk::ShaderModule,
    sampler: vk::Sampler,
    set_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
}

impl BlitUtils {
    fn new(utils: Weak<DeviceUtils>, device: Arc<DeviceFunctions>) -> Self {
        let vertex_shader = create_shader_from_bytes(&device, FULL_SCREEN_QUAD_VERTEX_SHADER).unwrap();
        let fragment_shader = create_shader_from_bytes(&device, BLIT_FRAGMENT_SHADER).unwrap();
        let sampler = Self::create_sampler(&device);
        let set_layout = Self::create_descriptor_set_layout(&device, sampler);
        let pipeline_layout = Self::create_pipeline_layout(&device, set_layout);

        Self {
            utils,
            device,
            vertex_shader,
            fragment_shader,
            sampler,
            set_layout,
            pipeline_layout
        }
    }

    pub fn create_blit_pass(&self, dst_format: vk::Format, load_op: vk::AttachmentLoadOp, initial_layout: vk::ImageLayout, final_layout: vk::ImageLayout) -> BlitPass {
        let render_pass = self.create_render_pass(dst_format, load_op, initial_layout, final_layout);
        let pipeline = self.create_pipeline(render_pass);

        BlitPass {
            utils: self.utils.upgrade().unwrap(),
            render_pass,
            pipeline,
        }
    }

    fn create_render_pass(&self, dst_format: vk::Format, load_op: vk::AttachmentLoadOp, initial_layout: vk::ImageLayout, final_layout: vk::ImageLayout) -> vk::RenderPass {
        let attachment = vk::AttachmentDescription::builder()
            .format(dst_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(load_op)
            .store_op(vk::AttachmentStoreOp::STORE)
            .initial_layout(initial_layout)
            .final_layout(final_layout);

        let attachment_reference = vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
        };

        let subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(std::slice::from_ref(&attachment_reference));

        let info = vk::RenderPassCreateInfo::builder()
            .attachments(std::slice::from_ref(&attachment))
            .subpasses(std::slice::from_ref(&subpass));

        unsafe {
            self.device.vk.create_render_pass(&info, None)
        }.unwrap()
    }

    fn create_pipeline(&self, render_pass: vk::RenderPass) -> vk::Pipeline {
        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(self.vertex_shader)
                .name(CStr::from_bytes_with_nul(b"main\0").unwrap())
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(self.fragment_shader)
                .name(CStr::from_bytes_with_nul(b"main\0").unwrap())
                .build()
        ];

        let input_state = vk::PipelineVertexInputStateCreateInfo::builder();

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_STRIP);

        let viewport = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .scissor_count(1);

        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false)
            .line_width(1.0);

        let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(false);

        let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(false)
            .depth_write_enable(false);

        let attachment = vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(false)
            .color_write_mask(vk::ColorComponentFlags::RGBA);

        let color_blend = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .attachments(std::slice::from_ref(&attachment));

        let dynamic_states = [
            vk::DynamicState::VIEWPORT,
            vk::DynamicState::SCISSOR
        ];

        let dynamic_state = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&dynamic_states);

        let info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&input_state)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .depth_stencil_state(&depth_stencil_state)
            .color_blend_state(&color_blend)
            .dynamic_state(&dynamic_state)
            .layout(self.pipeline_layout)
            .render_pass(render_pass);

        let pipeline = * unsafe {
            self.device.vk.create_graphics_pipelines(vk::PipelineCache::null(), std::slice::from_ref(&info), None)
        }.unwrap().get(0).unwrap();

        pipeline
    }

    fn create_sampler(device: &DeviceFunctions) -> vk::Sampler {
        let info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .anisotropy_enable(false)
            .compare_enable(false)
            .unnormalized_coordinates(false);

        unsafe {
            device.vk.create_sampler(&info, None)
        }.unwrap()
    }

    fn create_descriptor_set_layout(device: &DeviceFunctions, sampler: vk::Sampler) -> vk::DescriptorSetLayout {
        let binding = vk::DescriptorSetLayoutBinding::builder()
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .immutable_samplers(std::slice::from_ref(&sampler));

        let info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(std::slice::from_ref(&binding));

        unsafe {
            device.vk.create_descriptor_set_layout(&info, None)
        }.unwrap()
    }

    fn create_pipeline_layout(device: &DeviceFunctions, set_layout: vk::DescriptorSetLayout) -> vk::PipelineLayout {
        let info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(std::slice::from_ref(&set_layout));

        unsafe {
            device.vk.create_pipeline_layout(&info, None)
        }.unwrap()
    }
}

impl Drop for BlitUtils {
    fn drop(&mut self) {
        unsafe {
            self.device.vk.destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.vk.destroy_descriptor_set_layout(self.set_layout, None);
            self.device.vk.destroy_sampler(self.sampler, None);
            self.device.vk.destroy_shader_module(self.fragment_shader, None);
            self.device.vk.destroy_shader_module(self.vertex_shader, None);
        }
    }
}

pub struct BlitPass {
    utils: Arc<DeviceUtils>,
    render_pass: vk::RenderPass,
    pipeline: vk::Pipeline,
}

impl BlitPass {
    /// Allocates and writes descriptor sets for a collection of image views.
    ///
    /// The descriptor sets are fully owned by the calling code after this function returns.
    pub fn create_descriptor_sets(&self, pool: vk::DescriptorPool, image_views: &[vk::ImageView]) -> VkResult<Vec<vk::DescriptorSet>> {
        let layouts: Box<[_]> = repeat(self.utils.blit_utils.set_layout).take(image_views.len()).collect();

        let info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(pool)
            .set_layouts(layouts.as_ref());

        let sets = unsafe {
            self.utils.blit_utils.device.vk.allocate_descriptor_sets(&info)
        }?;

        let image_writes: Box<[_]> = image_views.iter().map(|view| {
            vk::DescriptorImageInfo::builder()
                .image_view(*view)
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        }).collect();

        let writes: Box<[_]> = sets.iter().zip(image_writes.iter()).map(|(set, info)| {
            vk::WriteDescriptorSet::builder()
                .dst_set(*set)
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(info))
                .build()
        }).collect();

        unsafe {
            self.utils.blit_utils.device.vk.update_descriptor_sets(writes.as_ref(), &[])
        };

        // We had to build so we need to make sure lifetimes are guaranteed
        drop(image_writes);

        Ok(sets)
    }

    /// Creates a framebuffer for a image view which can be used for this blit operation.
    ///
    /// The framebuffer is fully owned by the calling code and must be destroyed by it. In particular
    /// it must be guaranteed that the framebuffer is destroyed before this struct is dropped.
    pub fn create_framebuffer(&self, image_view: vk::ImageView, size: Vec2u32) -> VkResult<vk::Framebuffer> {
        let info = vk::FramebufferCreateInfo::builder()
            .render_pass(self.render_pass)
            .attachments(std::slice::from_ref(&image_view))
            .width(size[0])
            .height(size[1])
            .layers(1);

        unsafe {
            self.utils.blit_utils.device.vk.create_framebuffer(&info, None)
        }
    }

    /// Records a blit operation using a descriptor set and framebuffer previously created from this
    /// struct. No memory barriers are generated.
    ///
    /// The framebuffer image will be used in the COLOR_ATTACHMENT_OUTPUT stage and the sampled image
    /// in the FRAGMENT_SHADER stage. The sampled image must be in the SHADER_READ_OPTIMAL layout.
    pub fn record_blit(&self, command_buffer: vk::CommandBuffer, descriptor_set: vk::DescriptorSet, framebuffer: vk::Framebuffer, size: Vec2u32, clear_value: Option<&vk::ClearValue>) {
        let device = &self.utils.blit_utils.device;

        let mut info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_pass)
            .framebuffer(framebuffer)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D { width: size[0], height: size[1] }
            });

        if let Some(clear_value) = clear_value {
            info = info.clear_values(std::slice::from_ref(clear_value))
        }

        let viewport = vk::Viewport::builder()
            .x(0f32)
            .y(0f32)
            .width(size[0] as f32)
            .height(size[1] as f32)
            .min_depth(0.0)
            .max_depth(1.0);

        let scissor = vk::Rect2D {
            offset: vk::Offset2D{ x: 0, y: 0 },
            extent: vk::Extent2D{ width: size[0], height: size[1] }
        };

        unsafe {
            device.vk.cmd_set_viewport(command_buffer, 0, std::slice::from_ref(&viewport));
            device.vk.cmd_set_scissor(command_buffer, 0, std::slice::from_ref(&scissor));

            device.vk.cmd_begin_render_pass(command_buffer, &info, vk::SubpassContents::INLINE);

            device.vk.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline);

            device.vk.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.utils.blit_utils.pipeline_layout,
                0,
                std::slice::from_ref(&descriptor_set),
                &[]
            );

            device.vk.cmd_draw(command_buffer, 4, 1, 0, 0);

            device.vk.cmd_end_render_pass(command_buffer);
        }
    }

    pub fn get_device(&self) -> &Arc<DeviceFunctions> {
        &self.utils.blit_utils.device
    }
}

impl Drop for BlitPass {
    fn drop(&mut self) {
        unsafe {
            self.utils.blit_utils.device.vk.destroy_pipeline(self.pipeline, None);
            self.utils.blit_utils.device.vk.destroy_render_pass(self.render_pass, None);
        }
    }
}

static FULL_SCREEN_QUAD_VERTEX_SHADER: &'static [u8] = include_bytes_aligned!(4, concat!(env!("B4D_RESOURCE_DIR"), "utils/full_screen_quad_vert.spv"));
static BLIT_FRAGMENT_SHADER: &'static [u8] = include_bytes_aligned!(4, concat!(env!("B4D_RESOURCE_DIR"), "utils/blit_frag.spv"));