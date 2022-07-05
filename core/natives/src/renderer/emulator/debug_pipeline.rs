//! Provides a [`EmulatorPipeline`] implementation useful for debugging.

use std::collections::HashMap;
use std::ffi::CStr;
use std::sync::{Arc, Mutex, Weak};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Instant;
use ash::vk;
use bumpalo::Bump;
use bytemuck::{bytes_of, cast_slice, Pod, Zeroable};
use include_bytes_aligned::include_bytes_aligned;
use crate::device::device::Queue;
use crate::device::device_utils::create_shader_from_bytes;

use crate::prelude::*;
use crate::renderer::emulator::EmulatorRenderer;
use crate::renderer::emulator::mc_shaders::{McUniform, McUniformData, ShaderDropListener, ShaderId, ShaderListener, VertexFormat, VertexFormatEntry};
use crate::renderer::emulator::pipeline::{DrawTask, EmulatorPipeline, EmulatorPipelinePass, PipelineTask, PooledObjectProvider, SubmitRecorder};
use crate::util::vk::{make_full_rect, make_full_viewport};
use crate::vk::objects::allocator::{Allocation, AllocationStrategy};

pub struct DepthTypeInfo {
    pub vertex_stride: u32,
    pub vertex_position_offset: u32,
    pub vertex_position_format: vk::Format,
    pub topology: vk::PrimitiveTopology,
    pub primitive_restart: bool,
    pub discard: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ObjectCreateError {
    Vulkan(vk::Result),
    Allocation,
}

impl From<vk::Result> for ObjectCreateError {
    fn from(result: vk::Result) -> Self {
        Self::Vulkan(result)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum DebugPipelineMode {
    Depth,
    Position,
    Color,
    Normal,
    UV0,
    UV1,
    UV2,
    Textured0,
    Textured1,
    Textured2,
}

/// A [`EmulatorPipeline`] which provides debug information.
///
/// The following outputs are supported:
/// - Depth: The depth buffer
/// - Position: NDC coordinates of the pixel. (Not implemented yet)
/// - Color: The color vertex attribute
/// - Normal: The normal vertex attribute (Not implemented yet)
/// - UV0: The uv0 vertex attribute
/// - UV1: The uv1 vertex attribute
/// - UV2: The uv2 vertex attribute
/// - Textured0: The textured result from uv0 (Not implemented yet)
pub struct DebugPipeline {
    emulator: Arc<EmulatorRenderer>,
    weak: Weak<Self>,

    framebuffer_size: Vec2u32,

    shader_modules: ShaderModules,
    render_pass: vk::RenderPass,
    draw_pipeline: DrawPipeline,
    background_pipeline: BackgroundPipeline,
    descriptor_pool: vk::DescriptorPool,

    pipelines: Mutex<HashMap<ShaderId, ShaderPipelines>>,
    next_index: AtomicUsize,
    pass_objects: Box<[PassObjects]>,
    output_views: Box<[vk::ImageView]>,
}
assert_impl_all!(DebugPipeline: Send, Sync);

impl DebugPipeline {
    pub fn new(emulator: Arc<EmulatorRenderer>, mode: DebugPipelineMode, framebuffer_size: Vec2u32) -> Result<Arc<Self>, ObjectCreateError> {
        let concurrent_passes = 2usize;
        let depth_format = vk::Format::D32_SFLOAT;

        let device = emulator.get_device();

        let mut shader_modules = ShaderModules::new(device, mode)?;

        let render_pass = match Self::create_render_pass(&device, depth_format) {
            Ok(render_pass) => render_pass,
            Err(err) => {
                shader_modules.destroy(device);
                return Err(err);
            }
        };

        let mut draw_pipeline = match DrawPipeline::new(device) {
            Ok(pipeline) => pipeline,
            Err(err) => {
                unsafe { device.vk().destroy_render_pass(render_pass, None) };
                shader_modules.destroy(device);
                return Err(err);
            }
        };

        let mut background_pipeline = match BackgroundPipeline::new(device, render_pass, 1, framebuffer_size) {
            Ok(pipeline) => pipeline,
            Err(err) => {
                draw_pipeline.destroy(device);
                unsafe { device.vk().destroy_render_pass(render_pass, None) };
                shader_modules.destroy(device);
                return Err(err);
            }
        };

        let descriptor_pool = match Self::create_descriptor_pool(device, concurrent_passes) {
            Ok(pool) => pool,
            Err(err) => {
                background_pipeline.destroy(device);
                draw_pipeline.destroy(device);
                unsafe { device.vk().destroy_render_pass(render_pass, None) };
                shader_modules.destroy(device);
                return Err(err);
            }
        };

        let layouts: Box<[_]> = std::iter::repeat(background_pipeline.descriptor_set_layout).take(concurrent_passes).collect();
        let info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);

        let descriptor_sets = match unsafe {
            device.vk().allocate_descriptor_sets(&info)
        } {
            Ok(layouts) => layouts,
            Err(err) => {
                unsafe { device.vk().destroy_descriptor_pool(descriptor_pool, None) };
                background_pipeline.destroy(device);
                draw_pipeline.destroy(device);
                unsafe { device.vk().destroy_render_pass(render_pass, None) };
                shader_modules.destroy(device);
                return Err(ObjectCreateError::Vulkan(err));
            }
        };

        let mut pass_objects: Vec<PassObjects> = Vec::with_capacity(layouts.len());
        for descriptor_set in descriptor_sets {
            let objects = match PassObjects::new(device, framebuffer_size, depth_format, vk::Format::R8G8B8A8_SRGB, render_pass, descriptor_set) {
                Ok(objects) => objects,
                Err(err) => {
                    for mut pass_object in pass_objects {
                        pass_object.destroy(device);
                    }
                    unsafe { device.vk().destroy_descriptor_pool(descriptor_pool, None) };
                    background_pipeline.destroy(device);
                    draw_pipeline.destroy(device);
                    unsafe { device.vk().destroy_render_pass(render_pass, None) };
                    shader_modules.destroy(device);
                    return Err(err);
                }
            };
            pass_objects.push(objects);
        }
        let pass_objects = pass_objects.into_boxed_slice();

        let output_views: Box<_> = if mode == DebugPipelineMode::Depth {
            pass_objects.iter().map(|obj| obj.depth_sampler_view).collect()
        } else {
            pass_objects.iter().map(|obj| obj.output_view).collect()
        };

        Ok(Arc::new_cyclic(|weak| {
            Self {
                emulator,
                weak: weak.clone(),

                framebuffer_size,

                shader_modules,
                render_pass,
                draw_pipeline,
                background_pipeline,
                descriptor_pool,

                pipelines: Mutex::new(HashMap::new()),
                next_index: AtomicUsize::new(0),
                pass_objects,
                output_views
            }
        }))
    }

    /// Returns the next index to be used for a pass and increments the internal counter.
    fn next_index(&self) -> usize {
        loop {
            let current = self.next_index.load(Ordering::SeqCst);
            let next = (current + 1) % self.pass_objects.len();
            if self.next_index.compare_exchange(current, next, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                return current;
            }
        }
    }

    /// Returns the pipeline to be used for a specific configuration. If the pipeline doesnt exits
    /// yet a new one is created.
    fn get_pipeline(&self, shader: ShaderId, config: &PipelineConfig) -> vk::Pipeline {
        let mut guard = self.pipelines.lock().unwrap();
        let pipelines = guard.get_mut(&shader).unwrap_or_else(|| {
            log::error!("Called get_pipeline for unregistered shader {:?}", shader);
            panic!()
        });

        pipelines.get_or_create_pipeline(config, |format| self.create_pipeline(config, format))
    }

    fn create_pipeline(&self, config: &PipelineConfig, vertex_format: &VertexFormat) -> vk::Pipeline {
        let alloc = Bump::new();
        let (shader_stages, input_state) = self.shader_modules.configure_pipeline(vertex_format, &alloc);

        let viewport = make_full_viewport(self.framebuffer_size);
        let scissor = make_full_rect(self.framebuffer_size);

        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(std::slice::from_ref(&viewport))
            .scissors(std::slice::from_ref(&scissor));

        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .line_width(1f32);

        let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(false);

        let attachment_blend_state = [
            vk::PipelineColorBlendAttachmentState::builder()
                .blend_enable(true)
                .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
                .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                .color_blend_op(vk::BlendOp::ADD)
                .src_alpha_blend_factor(vk::BlendFactor::ONE)
                .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                .color_blend_op(vk::BlendOp::ADD)
                .color_write_mask(vk::ColorComponentFlags::RGBA)
                .build(),
        ];

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .attachments(&attachment_blend_state);

        let dynamic_state = vk::PipelineDynamicStateCreateInfo::builder();

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(config.primitive_topology)
            .primitive_restart_enable(false);

        let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(config.depth_test_enable)
            .depth_write_enable(config.depth_write_enable)
            .depth_compare_op(vk::CompareOp::LESS);

        let info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(shader_stages)
            .vertex_input_state(input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .depth_stencil_state(&depth_stencil_state)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state)
            .layout(self.draw_pipeline.pipeline_layout)
            .render_pass(self.render_pass)
            .subpass(0);

        let pipeline = *unsafe {
            self.emulator.get_device().vk().create_graphics_pipelines(vk::PipelineCache::null(), std::slice::from_ref(&info), None)
        }.unwrap_or_else(|(_, err)| {
            log::error!("Failed to create graphics pipeline {:?}", err);
            panic!();
        }).get(0).unwrap();

        pipeline
    }

    fn create_render_pass(device: &DeviceContext, depth_format: vk::Format) -> Result<vk::RenderPass, ObjectCreateError> {
        let attachments = [
            vk::AttachmentDescription::builder()
                .format(depth_format)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .build(),
            vk::AttachmentDescription::builder()
                .format(vk::Format::R8G8B8A8_SRGB)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::GENERAL)
                .build(),
            vk::AttachmentDescription::builder()
                .format(vk::Format::R8G8B8A8_SRGB)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::DONT_CARE)
                .store_op(vk::AttachmentStoreOp::STORE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .build()
        ];

        let pass_0_depth = vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
        };

        let pass_0_color = [
            vk::AttachmentReference {
                attachment: 1,
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
            },
        ];

        let pass_1_input = [
            vk::AttachmentReference {
                attachment: 1,
                layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
            },
        ];

        let pass_1_color = [
            vk::AttachmentReference {
                attachment: 2,
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
            },
        ];

        let subpasses = [
            vk::SubpassDescription::builder()
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .color_attachments(&pass_0_color)
                .depth_stencil_attachment(&pass_0_depth)
                .build(),
            vk::SubpassDescription::builder()
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .input_attachments(&pass_1_input)
                .color_attachments(&pass_1_color)
                .build(),
        ];

        let subpass_dependencies = [
            vk::SubpassDependency {
                src_subpass: 0,
                dst_subpass: 1,
                src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                dst_stage_mask: vk::PipelineStageFlags::FRAGMENT_SHADER,
                src_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                dst_access_mask: vk::AccessFlags::SHADER_READ,
                dependency_flags: vk::DependencyFlags::empty()
            }
        ];

        let info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&subpass_dependencies);

        let render_pass = unsafe {
            device.vk().create_render_pass(&info, None)
        }.map_err(|err| {
            log::error!("vkCreateRenderPass returned {:?} in DebugPipeline::create_render_pass", err);
            err
        })?;

        drop(pass_0_depth);
        drop(pass_0_color);
        drop(pass_1_input);
        drop(pass_1_color);

        Ok(render_pass)
    }

    fn create_descriptor_pool(device: &DeviceContext, concurrent_passes: usize) -> Result<vk::DescriptorPool, ObjectCreateError> {
        let concurrent_passes = concurrent_passes as u32;

        let sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::INPUT_ATTACHMENT,
                descriptor_count: concurrent_passes
            },
        ];

        let info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(concurrent_passes)
            .pool_sizes(&sizes);

        let descriptor_pool = unsafe {
            device.vk().create_descriptor_pool(&info, None)
        }.map_err(|err| {
            log::error!("vkCreateDescriptorPool returned {:?} in DebugPipeline::create_descriptor_pool", err);
            err
        })?;

        Ok(descriptor_pool)
    }
}

impl EmulatorPipeline for DebugPipeline {
    fn start_pass(&self) -> Box<dyn EmulatorPipelinePass + Send> {
        let index = self.next_index();
        self.pass_objects[index].wait_and_take();

        Box::new(DebugPipelinePass::new(self.weak.upgrade().unwrap(), index))
    }

    fn get_output(&self) -> (Vec2u32, &[vk::ImageView]) {
        (self.framebuffer_size, &self.output_views)
    }

    fn inc_shader_used(&self, shader: ShaderId) {
        let mut guard = self.pipelines.lock().unwrap();
        if let Some(pipelines) = guard.get_mut(&shader) {
            pipelines.inc_used();
        } else {
            let listener = self.emulator.get_shader(shader).unwrap_or_else(|| {
                log::error!("Called inc_shader_used for nonexistent shader {:?}", shader);
                panic!()
            }).register_drop_listener(&(self.weak.upgrade().unwrap() as Arc<dyn ShaderDropListener + Send + Sync>));

            let shader_obj = self.emulator.get_shader(shader).unwrap();
            let vertex_format = shader_obj.get_vertex_format().clone();
            let used_uniforms = shader_obj.get_used_uniforms();

            let mut  pipelines = ShaderPipelines::new(self.emulator.get_device().clone(), vertex_format, used_uniforms, listener);
            pipelines.inc_used();

            guard.insert(shader, pipelines);
        }
    }

    fn dec_shader_used(&self, shader: ShaderId) {
        let mut guard = self.pipelines.lock().unwrap();
        let pipelines = guard.get_mut(&shader).unwrap_or_else(|| {
            log::error!("Called dec_shader_used for shader which is not registered {:?}", shader);
            panic!();
        });
        pipelines.dec_used();
        let drop = pipelines.can_drop();
        if drop {
            guard.remove(&shader);
        }
    }
}

impl ShaderDropListener for DebugPipeline {
    fn on_shader_drop(&self, id: ShaderId) {
        let mut drop = false;
        let mut guard = self.pipelines.lock().unwrap();
        if let Some(pipeline) = guard.get_mut(&id) {
            pipeline.mark();
            drop = pipeline.can_drop();
        }
        if drop {
            guard.remove(&id);
        }
    }
}

impl Drop for DebugPipeline {
    fn drop(&mut self) {
        let device = self.emulator.get_device();
        for objects in self.pass_objects.iter_mut() {
            objects.destroy(device);
        }
        self.pipelines.get_mut().unwrap().clear();
        unsafe {
            device.vk().destroy_descriptor_pool(self.descriptor_pool, None);
        }
        self.background_pipeline.destroy(device);
        self.draw_pipeline.destroy(device);
        unsafe {
            device.vk().destroy_render_pass(self.render_pass, None);
        }
        self.shader_modules.destroy(device);
    }
}

/// The shader modules needed to create vulkan pipelines for the debug pipeline
struct ShaderModules {
    mode: DebugPipelineMode,
    vertex_module: vk::ShaderModule,
    null_module: vk::ShaderModule,
    fragment_module: vk::ShaderModule,
    texture_module: Option<vk::ShaderModule>,
}

impl ShaderModules {
    fn new(device: &DeviceContext, mode: DebugPipelineMode) -> Result<Self, ObjectCreateError> {
        let null_module = try_create_shader_module(device, DEBUG_NULL_VERTEX_BIN, "null_vertex")?;

        let fragment_module = try_create_shader_module(device, DEBUG_FRAGMENT_BIN, "fragment").map_err(|err| {
            unsafe { device.vk().destroy_shader_module(null_module, None) };
            err
        })?;

        let vertex_module = match mode {
            DebugPipelineMode::Depth => try_create_shader_module(device, DEBUG_POSITION_VERTEX_BIN, "position_vertex"),
            DebugPipelineMode::Position => try_create_shader_module(device, DEBUG_POSITION_VERTEX_BIN, "position_vertex"),
            DebugPipelineMode::Color => try_create_shader_module(device, DEBUG_COLOR_VERTEX_BIN, "color_vertex"),
            DebugPipelineMode::Normal => { todo!() }
            DebugPipelineMode::UV0 |
            DebugPipelineMode::UV1 |
            DebugPipelineMode::UV2 |
            DebugPipelineMode::Textured0 |
            DebugPipelineMode::Textured1 |
            DebugPipelineMode::Textured2 => try_create_shader_module(device, DEBUG_UV_VERTEX_BIN, "uv_vertex"),
        }.map_err(|err| {
            unsafe {
                device.vk().destroy_shader_module(null_module, None);
                device.vk().destroy_shader_module(fragment_module, None);
            }
            err
        })?;

        let texture_module = match mode {
            DebugPipelineMode::Textured0 => try_create_shader_module(device, TEXTURED_FRAGMENT_BIN, "textured_fragment").map(|val| Some(val)),
            _ => Ok(None),
        }.map_err(|err| {
            unsafe {
                device.vk().destroy_shader_module(null_module, None);
                device.vk().destroy_shader_module(fragment_module, None);
                device.vk().destroy_shader_module(vertex_module, None);
            }
            err
        })?;

        Ok(Self {
            mode,
            vertex_module,
            null_module,
            fragment_module,
            texture_module,
        })
    }

    fn configure_pipeline<'s, 'a: 's>(&'s self, vertex_format: &VertexFormat, alloc: &'a Bump) -> (&'a [vk::PipelineShaderStageCreateInfo], &'a vk::PipelineVertexInputStateCreateInfo) {
        let input_bindings: &[_] = alloc.alloc([
            vk::VertexInputBindingDescription {
                binding: 0,
                stride: vertex_format.stride,
                input_rate: vk::VertexInputRate::VERTEX
            }
        ]);

        let vertex_module;
        let input_attributes: &[_];
        let vertex_format_supported;
        if let Some(entry) = self.process_vertex_format(vertex_format) {
            vertex_format_supported = true;
            vertex_module = self.vertex_module;

            input_attributes = alloc.alloc([
                vk::VertexInputAttributeDescription {
                    location: 0,
                    binding: 0,
                    format: vertex_format.position.format,
                    offset: vertex_format.position.offset,
                },
                vk::VertexInputAttributeDescription {
                    location: 1,
                    binding: 0,
                    format: entry.format,
                    offset: entry.offset
                }
            ]);
        } else {
            vertex_format_supported = false;
            vertex_module = self.null_module;

            input_attributes = alloc.alloc([
                vk::VertexInputAttributeDescription {
                    location: 0,
                    binding: 0,
                    format: vertex_format.position.format,
                    offset: vertex_format.position.offset,
                },
            ]);
        }

        let (fragment_module, fragment_specialization) = match (self.mode, vertex_format_supported) {
            (DebugPipelineMode::Textured0, true) |
            (DebugPipelineMode::Textured1, true) |
            (DebugPipelineMode::Textured2, true) => {
                let data = alloc.alloc(match self.mode {
                    DebugPipelineMode::Textured0 => 0u32,
                    DebugPipelineMode::Textured1 => 1u32,
                    DebugPipelineMode::Textured2 => 2u32,
                    _ => panic!(),
                });
                let entries = alloc.alloc([
                    vk::SpecializationMapEntry {
                        constant_id: 0,
                        offset: 0,
                        size: 4
                    }
                ]);
                (*self.texture_module.as_ref().unwrap(), alloc.alloc(vk::SpecializationInfo::builder()
                    .map_entries(entries)
                    .data(bytes_of(data))
                ))
            }
            _ => {
                (self.fragment_module, alloc.alloc(vk::SpecializationInfo::builder()))
            }
        };

        let shader_stages: &[_] = alloc.alloc([
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vertex_module)
                .name(SHADER_ENTRY)
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(fragment_module)
                .name(SHADER_ENTRY)
                .specialization_info(fragment_specialization)
                .build(),
        ]);

        let input_state: &_ = alloc.alloc(vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(input_bindings)
            .vertex_attribute_descriptions(input_attributes)
            .build()
        );

        (shader_stages, input_state)
    }

    fn process_vertex_format<'a>(&self, vertex_format: &'a VertexFormat) -> Option<&'a VertexFormatEntry> {
        match self.mode {
            DebugPipelineMode::Depth |
            DebugPipelineMode::Position => Some(&vertex_format.position),
            DebugPipelineMode::Color => vertex_format.color.as_ref(),
            DebugPipelineMode::Normal => vertex_format.normal.as_ref(),
            DebugPipelineMode::UV0 |
            DebugPipelineMode::Textured0 => vertex_format.uv0.as_ref(),
            DebugPipelineMode::UV1 |
            DebugPipelineMode::Textured1 => vertex_format.uv1.as_ref(),
            DebugPipelineMode::UV2 |
            DebugPipelineMode::Textured2 => vertex_format.uv2.as_ref(),
        }
    }

    fn destroy(&mut self, device: &DeviceContext) {
        unsafe {
            device.vk().destroy_shader_module(self.vertex_module, None);
            device.vk().destroy_shader_module(self.null_module, None);
            device.vk().destroy_shader_module(self.fragment_module, None);
            if let Some(texture_module) = self.texture_module.take() {
                device.vk().destroy_shader_module(texture_module, None);
            }
        }
    }
}

struct DrawPipeline {
    set0_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
}

impl DrawPipeline {
    fn new(device: &DeviceContext) -> Result<Self, ObjectCreateError> {
        let bindings = [
            vk::DescriptorSetLayoutBinding {
                binding: 0,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::ALL,
                p_immutable_samplers: std::ptr::null(),
            },
            vk::DescriptorSetLayoutBinding {
                binding: 1,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 3,
                stage_flags: vk::ShaderStageFlags::ALL_GRAPHICS,
                p_immutable_samplers: std::ptr::null(),
            },
        ];

        let info = vk::DescriptorSetLayoutCreateInfo::builder()
            .flags(vk::DescriptorSetLayoutCreateFlags::PUSH_DESCRIPTOR_KHR)
            .bindings(&bindings);

        let set0_layout = unsafe {
            device.vk().create_descriptor_set_layout(&info, None)
        }.map_err(|err| {
            log::error!("vkCreateDescriptorSetLayout returned {:?} in DrawPipeline::new when creating set 0 layout", err);
            err
        })?;

        let push_constant_range = vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::ALL_GRAPHICS,
            offset: 0,
            size: std::mem::size_of::<PushConstants>() as u32,
        };

        let layouts = [
            set0_layout
        ];

        let info = vk::PipelineLayoutCreateInfo::builder()
            .push_constant_ranges(std::slice::from_ref(&push_constant_range))
            .set_layouts(&layouts);

        let pipeline_layout = unsafe {
            device.vk().create_pipeline_layout(&info, None)
        }.map_err(|err| {
            log::error!("vkCreatePipelineLayout returned {:?} in DrawPipeline::new", err);
            unsafe { device.vk().destroy_descriptor_set_layout(set0_layout, None) };
            err
        })?;

        Ok(Self {
            set0_layout,
            pipeline_layout
        })
    }

    fn destroy(&mut self, device: &DeviceContext) {
        unsafe {
            device.vk().destroy_pipeline_layout(self.pipeline_layout, None);
            device.vk().destroy_descriptor_set_layout(self.set0_layout, None);
        }
    }
}

struct BackgroundPipeline {
    descriptor_set_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
}

impl BackgroundPipeline {
    fn new(device: &DeviceContext, render_pass: vk::RenderPass, subpass: u32, framebuffer_size: Vec2u32) -> Result<Self, ObjectCreateError> {
        let bindings = [
            vk::DescriptorSetLayoutBinding {
                binding: 0,
                descriptor_type: vk::DescriptorType::INPUT_ATTACHMENT,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                p_immutable_samplers: std::ptr::null()
            },
        ];

        let info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&bindings);

        let descriptor_set_layout = unsafe {
            device.vk().create_descriptor_set_layout(&info, None)
        }.map_err(|err| {
            log::error!("vkCreateDescriptorSetLayout returned {:?} in BackgroundPipeline::new", err);
            err
        })?;

        let info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(std::slice::from_ref(&descriptor_set_layout));

        let pipeline_layout = unsafe {
            device.vk().create_pipeline_layout(&info, None)
        }.map_err(|err| {
            log::error!("vkCreatePipelineLayout returned {:?} in BackgroundPipeline::new", err);
            unsafe { device.vk().destroy_descriptor_set_layout(descriptor_set_layout, None) };
            err
        })?;

        let pipeline = Self::create_pipeline(device, pipeline_layout, render_pass, subpass, framebuffer_size).map_err(|err| {
            unsafe {
                device.vk().destroy_pipeline_layout(pipeline_layout, None);
                device.vk().destroy_descriptor_set_layout(descriptor_set_layout, None);
            }
            err
        })?;

        Ok(Self {
            descriptor_set_layout,
            pipeline_layout,
            pipeline
        })
    }

    fn destroy(&mut self, device: &DeviceContext) {
        unsafe {
            device.vk().destroy_pipeline(self.pipeline, None);
            device.vk().destroy_pipeline_layout(self.pipeline_layout, None);
            device.vk().destroy_descriptor_set_layout(self.descriptor_set_layout, None);
        }
    }

    fn create_pipeline(device: &DeviceContext, layout: vk::PipelineLayout, render_pass: vk::RenderPass, subpass: u32, framebuffer_size: Vec2u32) -> Result<vk::Pipeline, ObjectCreateError> {
        let vertex_module = try_create_shader_module(device, BACKGROUND_VERTEX_BIN, "background_vert")?;
        let fragment_module = try_create_shader_module(device, BACKGROUND_FRAGMENT_BIN, "background_frag").map_err(|err| {
            unsafe { device.vk().destroy_shader_module(vertex_module, None) };
            err
        })?;

        let specialization_data = Vec2f32::new(framebuffer_size[0] as f32, framebuffer_size[1] as f32);
        let specializations = [
            vk::SpecializationMapEntry {
                constant_id: 0,
                offset: 0,
                size: 4
            },
            vk::SpecializationMapEntry {
                constant_id: 1,
                offset: 4,
                size: 4
            }
        ];

        let specialization_info = vk::SpecializationInfo::builder()
            .map_entries(&specializations)
            .data(cast_slice(specialization_data.data.as_slice()));

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vertex_module)
                .name(SHADER_ENTRY)
                .specialization_info(&specialization_info)
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(fragment_module)
                .name(SHADER_ENTRY)
                .build()
        ];

        let input_state = vk::PipelineVertexInputStateCreateInfo::builder();

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_STRIP)
            .primitive_restart_enable(false);

        let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(false)
            .depth_write_enable(false)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false);

        let viewport = make_full_viewport(framebuffer_size);
        let scissor = make_full_rect(framebuffer_size);

        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(std::slice::from_ref(&viewport))
            .scissors(std::slice::from_ref(&scissor));

        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::CLOCKWISE)
            .line_width(1f32);

        let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(false);

        let attachment_blend_state = [
            vk::PipelineColorBlendAttachmentState::builder()
                .blend_enable(false)
                .color_write_mask(vk::ColorComponentFlags::RGBA)
                .build()
        ];

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .attachments(&attachment_blend_state);

        let dynamic_state = vk::PipelineDynamicStateCreateInfo::builder();

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
            .layout(layout)
            .render_pass(render_pass)
            .subpass(subpass);

        let pipeline = *unsafe {
            device.vk().create_graphics_pipelines(vk::PipelineCache::null(), std::slice::from_ref(&info), None)
        }.map_err(|(_, err)| {
            log::error!("vkCreateGraphicsPipelines returned {:?} in BackgroundPipeline::create_pipeline", err);
            unsafe {
                device.vk().destroy_shader_module(vertex_module, None);
                device.vk().destroy_shader_module(fragment_module, None);
            }
            err
        })?.get(0).unwrap();

        unsafe {
            device.vk().destroy_shader_module(vertex_module, None);
            device.vk().destroy_shader_module(fragment_module, None);
        }
        drop(specialization_info);

        Ok(pipeline)
    }
}

struct PassObjects {
    ready: AtomicBool,

    depth_image: vk::Image,
    depth_framebuffer_view: vk::ImageView,
    depth_sampler_view: vk::ImageView,

    pass_image: vk::Image,
    pass_view: vk::ImageView,

    output_image: vk::Image,
    output_view: vk::ImageView,

    bg_descriptor_set: vk::DescriptorSet,
    framebuffer: vk::Framebuffer,

    allocations: Vec<Allocation>,
}

impl PassObjects {
    fn new(device: &DeviceContext, framebuffer_size: Vec2u32, depth_format: vk::Format, color_format: vk::Format, render_pass: vk::RenderPass, bg_descriptor_set: vk::DescriptorSet) -> Result<Self, ObjectCreateError> {
        let mut result = PassObjects {
            ready: AtomicBool::new(true),

            depth_image: vk::Image::null(),
            depth_framebuffer_view: vk::ImageView::null(),
            depth_sampler_view: vk::ImageView::null(),

            pass_image: vk::Image::null(),
            pass_view: vk::ImageView::null(),

            output_image: vk::Image::null(),
            output_view: vk::ImageView::null(),

            bg_descriptor_set,
            framebuffer: vk::Framebuffer::null(),

            allocations: Vec::with_capacity(3)
        };

        let (depth_image, allocation) = Self::create_image(device, framebuffer_size, depth_format, vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::SAMPLED)?;
        result.depth_image = depth_image;
        result.allocations.push(allocation);

        let depth_framebuffer_view = Self::create_image_view(device, depth_image, depth_format, vk::ImageAspectFlags::DEPTH, false).map_err(|err| {
            result.destroy(device);
            err
        })?;
        result.depth_framebuffer_view = depth_framebuffer_view;

        let depth_sampler_view = Self::create_image_view(device, depth_image, depth_format, vk::ImageAspectFlags::DEPTH, true).map_err(|err| {
            result.destroy(device);
            err
        })?;
        result.depth_sampler_view = depth_sampler_view;

        let (pass_image, allocation) = Self::create_image(device, framebuffer_size, color_format, vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::INPUT_ATTACHMENT).map_err(|err| {
            result.destroy(device);
            err
        })?;
        result.pass_image = pass_image;
        result.allocations.push(allocation);

        let pass_view = Self::create_image_view(device, pass_image, color_format, vk::ImageAspectFlags::COLOR, false).map_err(|err| {
            result.destroy(device);
            err
        })?;
        result.pass_view = pass_view;

        let (output_image, allocation) = Self::create_image(device, framebuffer_size, color_format, vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED).map_err(|err| {
            result.destroy(device);
            err
        })?;
        result.output_image = output_image;
        result.allocations.push(allocation);

        let output_view = Self::create_image_view(device, output_image, color_format, vk::ImageAspectFlags::COLOR, false).map_err(|err| {
            result.destroy(device);
            err
        })?;
        result.output_view = output_view;

        let framebuffer = Self::create_framebuffer(device, framebuffer_size, depth_framebuffer_view, pass_view, output_view, render_pass).map_err(|err| {
            result.destroy(device);
            err
        })?;
        result.framebuffer = framebuffer;

        let info = vk::DescriptorImageInfo::builder()
            .image_view(pass_view)
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);

        let write = vk::WriteDescriptorSet::builder()
            .dst_set(bg_descriptor_set)
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
            .image_info(std::slice::from_ref(&info));

        unsafe {
            device.vk().update_descriptor_sets(std::slice::from_ref(&write), &[])
        };

        Ok(result)
    }

    fn wait_and_take(&self) {
        let mut start = Instant::now();
        loop {
            if self.ready.compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                return;
            }
            std::thread::yield_now();
            if start.elapsed().as_millis() > 1000 {
                log::warn!("Hit 1s timeout waiting for next debug pipeline object");
                start = Instant::now();
            }
        }
    }

    fn destroy(&mut self, device: &DeviceContext) {
        unsafe {
            if self.framebuffer != vk::Framebuffer::null() {
                device.vk().destroy_framebuffer(self.framebuffer, None);
            }
            if self.output_view != vk::ImageView::null() {
                device.vk().destroy_image_view(self.output_view, None);
            }
            if self.output_image != vk::Image::null() {
                device.vk().destroy_image(self.output_image, None);
            }
            if self.pass_view != vk::ImageView::null() {
                device.vk().destroy_image_view(self.pass_view, None);
            }
            if self.pass_image != vk::Image::null() {
                device.vk().destroy_image(self.pass_image, None);
            }
            if self.depth_sampler_view != vk::ImageView::null() {
                device.vk().destroy_image_view(self.depth_sampler_view, None);
            }
            if self.depth_framebuffer_view != vk::ImageView::null() {
                device.vk().destroy_image_view(self.depth_framebuffer_view, None);
            }
            if self.depth_image != vk::Image::null() {
                device.vk().destroy_image(self.depth_image, None);
            }
        }
        for allocation in std::mem::replace(&mut self.allocations, Vec::new()) {
            device.get_allocator().free(allocation);
        }
    }

    fn create_image(device: &DeviceContext, size: Vec2u32, format: vk::Format, usage: vk::ImageUsageFlags) -> Result<(vk::Image, Allocation), ObjectCreateError> {
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
        }.map_err(|err| {
            log::error!("vkCreateImage returned {:?} in PassObjects::create_image", err);
            err
        })?;

        let allocation = device.get_allocator().allocate_image_memory(image, &AllocationStrategy::AutoGpuOnly).map_err(|err| {
            log::error!("Failed to allocate image memory in PassObjects::create_image {:?}", err);
            unsafe { device.vk().destroy_image(image, None) };
            ObjectCreateError::Allocation
        })?;

        if let Err(err) = unsafe {
            device.vk().bind_image_memory(image, allocation.memory(), allocation.offset())
        } {
            log::error!("Failed to bind image memory in PassObjects::create_image {:?}", err);
            unsafe { device.vk().destroy_image(image, None) };
            device.get_allocator().free(allocation);
            return Err(ObjectCreateError::Vulkan(err));
        }

        Ok((image, allocation))
    }

    fn create_image_view(device: &DeviceContext, image: vk::Image, format: vk::Format, aspect_mask: vk::ImageAspectFlags, swizzle_r: bool) -> Result<vk::ImageView, ObjectCreateError> {
        let info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format);

        let info = if swizzle_r {
            info.components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::R,
                g: vk::ComponentSwizzle::R,
                b: vk::ComponentSwizzle::R,
                a: vk::ComponentSwizzle::ONE
            })
        } else {
            info.components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY
            })
        };

        let info = info
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1
            });

        let image_view = unsafe {
            device.vk().create_image_view(&info, None)
        }.map_err(|err| {
            log::error!("vkCreateImageView returned {:?} in PassObjects::create_image_view", err);
            err
        })?;

        Ok(image_view)
    }

    fn create_framebuffer(device: &DeviceContext, size: Vec2u32, depth_view: vk::ImageView, pass_view: vk::ImageView, output_view: vk::ImageView, render_pass: vk::RenderPass) -> Result<vk::Framebuffer, ObjectCreateError> {
        let attachments = [
            depth_view, pass_view, output_view
        ];

        let info = vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(&attachments)
            .width(size[0])
            .height(size[1])
            .layers(1);

        let framebuffer = unsafe {
            device.vk().create_framebuffer(&info, None)
        }.map_err(|err| {
            log::error!("vkCreateFramebuffer returned {:?} in PassObjects::create_framebuffer", err);
            err
        })?;

        Ok(framebuffer)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
struct PipelineConfig {
    primitive_topology: vk::PrimitiveTopology,
    depth_test_enable: bool,
    depth_write_enable: bool,
}

struct ShaderPipelines {
    device: Arc<DeviceContext>,
    vertex_format: VertexFormat,
    used_uniforms: McUniform,
    pipelines: HashMap<PipelineConfig, vk::Pipeline>,
    #[allow(unused)]
    listener: ShaderListener,
    used_counter: u32,
    marked: bool,
}

impl ShaderPipelines {
    fn new(device: Arc<DeviceContext>, vertex_format: VertexFormat, used_uniforms: McUniform, listener: ShaderListener) -> Self {
        Self {
            device,
            vertex_format,
            used_uniforms,
            pipelines: HashMap::new(),
            listener,
            used_counter: 0,
            marked: false,
        }
    }

    fn get_or_create_pipeline<T: FnOnce(&VertexFormat) -> vk::Pipeline>(&mut self, config: &PipelineConfig, create_fn: T) -> vk::Pipeline {
        if let Some(pipeline) = self.pipelines.get(config) {
            *pipeline
        } else {
            let pipeline = create_fn(&self.vertex_format);
            self.pipelines.insert(*config, pipeline);
            pipeline
        }
    }

    fn inc_used(&mut self) {
        self.used_counter += 1;
    }

    fn dec_used(&mut self) {
        self.used_counter -= 1;
    }

    fn mark(&mut self) {
        self.marked = true;
    }

    fn can_drop(&self) -> bool {
        self.marked && self.used_counter == 0
    }
}

impl Drop for ShaderPipelines {
    fn drop(&mut self) {
        for pipeline in self.pipelines.values() {
            unsafe {
                self.device.vk().destroy_pipeline(*pipeline, None);
            }
        }
    }
}

struct DebugPipelinePass {
    parent: Arc<DebugPipeline>,
    index: usize,

    placeholder_texture: vk::ImageView,
    placeholder_sampler: vk::Sampler,
    shader_uniforms: HashMap<ShaderId, UniformStateTracker>,

    command_buffer: Option<vk::CommandBuffer>,
    current_pipeline: Option<(ShaderId, PipelineConfig)>,
    current_vertex_buffer: Option<vk::Buffer>,
    current_index_buffer: Option<vk::Buffer>,
}

impl DebugPipelinePass {
    fn new(parent: Arc<DebugPipeline>, index: usize) -> Self {
        Self {
            parent,
            index,

            placeholder_texture: vk::ImageView::null(),
            placeholder_sampler: vk::Sampler::null(),
            shader_uniforms: HashMap::new(),

            command_buffer: None,
            current_pipeline: None,
            current_vertex_buffer: None,
            current_index_buffer: None
        }
    }

    fn update_uniform(&mut self, shader: ShaderId, data: &McUniformData) {
        if !self.shader_uniforms.contains_key(&shader) {
            let uniforms = self.parent.pipelines.lock().unwrap().get(&shader).unwrap().used_uniforms;
            self.shader_uniforms.insert(shader, UniformStateTracker::new(uniforms, self.placeholder_texture, self.placeholder_sampler));
        }
        let tracker = self.shader_uniforms.get_mut(&shader).unwrap();
        tracker.update_uniform(data);
    }

    fn update_texture(&mut self, shader: ShaderId, index: u32, view: vk::ImageView, sampler: vk::Sampler) {
        if !self.shader_uniforms.contains_key(&shader) {
            let uniforms = self.parent.pipelines.lock().unwrap().get(&shader).unwrap().used_uniforms;
            self.shader_uniforms.insert(shader, UniformStateTracker::new(uniforms, self.placeholder_texture, self.placeholder_sampler));
        }
        let tracker = self.shader_uniforms.get_mut(&shader).unwrap();
        tracker.update_texture(index, view, sampler);
    }

    fn draw(&mut self, task: &DrawTask, obj: &mut PooledObjectProvider) {
        let device = self.parent.emulator.get_device();
        let cmd = *self.command_buffer.as_ref().unwrap();

        let pipeline_config = PipelineConfig {
            primitive_topology: task.primitive_topology,
            depth_test_enable: true,
            depth_write_enable: task.depth_write_enable
        };

        if self.current_pipeline != Some((task.shader, pipeline_config)) {
            self.current_pipeline = Some((task.shader, pipeline_config));

            let new_pipeline = self.parent.get_pipeline(task.shader, &pipeline_config);
            unsafe {
                device.vk().cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, new_pipeline);
            }
        }

        if !self.shader_uniforms.contains_key(&task.shader) {
            log::warn!("Called draw without any shader uniforms. Using default values!");
            let uniforms = self.parent.pipelines.lock().unwrap().get(&task.shader).unwrap().used_uniforms;
            self.shader_uniforms.insert(task.shader, UniformStateTracker::new(uniforms, self.placeholder_texture, self.placeholder_sampler));
        }
        if let Some(tracker) = self.shader_uniforms.get_mut(&task.shader) {
            if let Some(push_constants) = tracker.validate_push_constants() {
                unsafe {
                    device.vk().cmd_push_constants(
                        self.command_buffer.unwrap(),
                        self.parent.draw_pipeline.pipeline_layout,
                        vk::ShaderStageFlags::ALL_GRAPHICS,
                        0,
                        bytes_of(push_constants)
                    );
                }
            }

            if let Some(static_uniforms) = tracker.validate_static_uniforms() {
                let (buffer, offset) = obj.allocate_uniform(bytes_of(static_uniforms));
                let buffer_info = vk::DescriptorBufferInfo {
                    buffer,
                    offset,
                    range: std::mem::size_of::<StaticUniforms>() as vk::DeviceSize
                };
                let write = vk::WriteDescriptorSet::builder()
                    .dst_binding(0)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .buffer_info(std::slice::from_ref(&buffer_info));

                unsafe {
                    device.push_descriptor_khr().cmd_push_descriptor_set(
                        self.command_buffer.unwrap(),
                        vk::PipelineBindPoint::GRAPHICS,
                        self.parent.draw_pipeline.pipeline_layout,
                        0,
                        std::slice::from_ref(&write)
                    );
                }
            }

            if let Some(textures) = tracker.validate_textures() {
                let image_info0 = vk::DescriptorImageInfo {
                    sampler: textures[0].1,
                    image_view: textures[0].0,
                    image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
                };
                let image_info1 = vk::DescriptorImageInfo {
                    sampler: textures[1].1,
                    image_view: textures[1].0,
                    image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
                };
                let image_info2 = vk::DescriptorImageInfo {
                    sampler: textures[2].1,
                    image_view: textures[2].0,
                    image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
                };
                let writes = [
                    vk::WriteDescriptorSet::builder()
                        .dst_binding(1)
                        .dst_array_element(0)
                        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                        .image_info(std::slice::from_ref(&image_info0))
                        .build(),
                    vk::WriteDescriptorSet::builder()
                        .dst_binding(1)
                        .dst_array_element(1)
                        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                        .image_info(std::slice::from_ref(&image_info1))
                        .build(),
                    vk::WriteDescriptorSet::builder()
                        .dst_binding(1)
                        .dst_array_element(2)
                        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                        .image_info(std::slice::from_ref(&image_info2))
                        .build(),
                ];

                unsafe {
                    device.push_descriptor_khr().cmd_push_descriptor_set(
                        self.command_buffer.unwrap(),
                        vk::PipelineBindPoint::GRAPHICS,
                        self.parent.draw_pipeline.pipeline_layout,
                        0,
                        &writes
                    );
                }
            }
        }

        if self.current_vertex_buffer != Some(task.vertex_buffer) {
            unsafe {
                device.vk().cmd_bind_vertex_buffers(
                    cmd,
                    0,
                    std::slice::from_ref(&task.vertex_buffer),
                    std::slice::from_ref(&0)
                );
            }
            self.current_vertex_buffer = Some(task.vertex_buffer);
        }

        if self.current_index_buffer != Some(task.index_buffer) {
            unsafe {
                device.vk().cmd_bind_index_buffer(cmd, task.index_buffer, 0, task.index_type);
            }
            self.current_index_buffer = Some(task.index_buffer);
        }

        unsafe {
            device.vk().cmd_draw_indexed(cmd, task.index_count, 1, task.first_index, task.vertex_offset, 0);
        }
    }
}

impl EmulatorPipelinePass for DebugPipelinePass {
    fn init(&mut self, _: &Queue, obj: &mut PooledObjectProvider, placeholder_texture: vk::ImageView, placeholder_sampler: vk::Sampler) {
        self.placeholder_texture = placeholder_texture;
        self.placeholder_sampler = placeholder_sampler;

        let cmd = obj.get_begin_command_buffer().unwrap();
        self.command_buffer = Some(cmd);

        let device = self.parent.emulator.get_device();

        let clear_values = [
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0
                }
            },
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0f32, 0f32, 0f32, 0f32],
                }
            },
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0f32, 0f32, 0f32, 0f32],
                }
            }
        ];
        let info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.parent.render_pass)
            .framebuffer(self.parent.pass_objects[self.index].framebuffer)
            .render_area(make_full_rect(self.parent.framebuffer_size))
            .clear_values(&clear_values);

        unsafe {
            device.vk().cmd_begin_render_pass(cmd, &info, vk::SubpassContents::INLINE);
        }
    }

    fn process_task(&mut self, task: &PipelineTask, obj: &mut PooledObjectProvider) {
        match task {
            PipelineTask::UpdateUniform(shader, data) => {
                self.update_uniform(*shader, data);
            }
            PipelineTask::UpdateTexture(shader, index, view, sampler) => {
                self.update_texture(*shader, *index, *view, *sampler);
            }
            PipelineTask::Draw(task) => {
                self.draw(task, obj);
            }
        }
    }

    fn record<'a>(&mut self, _: &mut PooledObjectProvider, submits: &mut SubmitRecorder<'a>, alloc: &'a Bump) {
        let device = self.parent.emulator.get_device();
        let cmd = self.command_buffer.take().unwrap();

        let bg_descriptor_sets = [self.parent.pass_objects[self.index].bg_descriptor_set];

        unsafe {
            device.vk().cmd_next_subpass(cmd, vk::SubpassContents::INLINE);
            device.vk().cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, self.parent.background_pipeline.pipeline);
            device.vk().cmd_bind_descriptor_sets(cmd, vk::PipelineBindPoint::GRAPHICS, self.parent.background_pipeline.pipeline_layout, 0, &bg_descriptor_sets, &[]);
            device.vk().cmd_draw(cmd, 4, 1, 0, 0);
        }

        let image_barrier = [
            vk::ImageMemoryBarrier2::builder()
                .src_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
                .src_access_mask(vk::AccessFlags2::COLOR_ATTACHMENT_WRITE)
                .dst_stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
                .dst_access_mask(vk::AccessFlags2::MEMORY_READ)
                .old_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .src_queue_family_index(0)
                .dst_queue_family_index(0)
                .image(self.parent.pass_objects[self.index].depth_image)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::DEPTH,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1
                })
                .build(),
            vk::ImageMemoryBarrier2::builder()
                .src_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
                .src_access_mask(vk::AccessFlags2::COLOR_ATTACHMENT_WRITE)
                .dst_stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
                .dst_access_mask(vk::AccessFlags2::MEMORY_READ)
                .old_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .src_queue_family_index(0)
                .dst_queue_family_index(0)
                .image(self.parent.pass_objects[self.index].output_image)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1
                })
                .build(),
        ];

        let info = vk::DependencyInfo::builder()
            .image_memory_barriers(&image_barrier);

        unsafe {
            device.vk().cmd_end_render_pass(cmd);

            device.synchronization_2_khr().cmd_pipeline_barrier2(cmd, &info);

            device.vk().end_command_buffer(cmd).unwrap();
        }

        let command_buffer_info = alloc.alloc(vk::CommandBufferSubmitInfo::builder()
            .command_buffer(cmd)
        );

        submits.push(vk::SubmitInfo2::builder()
            .command_buffer_infos(std::slice::from_ref(command_buffer_info))
        );
    }

    fn get_output_index(&self) -> usize {
        self.index
    }

    fn get_internal_fences(&self, _: &mut Vec<vk::Fence>) {
        todo!()
    }
}

impl Drop for DebugPipelinePass {
    fn drop(&mut self) {
        self.parent.pass_objects[self.index].ready.store(true, Ordering::SeqCst);
    }
}

struct UniformStateTracker {
    used_uniforms: McUniform,
    push_constants_dirty: bool,
    static_uniforms_dirty: bool,
    textures_dirty: bool,
    push_constant_cache: PushConstants,
    static_uniform_cache: StaticUniforms,
    textures: [(vk::ImageView, vk::Sampler); 3],
}

impl UniformStateTracker {
    fn new(used_uniforms: McUniform, initial_texture: vk::ImageView, initial_sampler: vk::Sampler) -> Self {
        Self {
            used_uniforms,
            push_constants_dirty: true,
            static_uniforms_dirty: true,
            textures_dirty: true,
            push_constant_cache: PushConstants {
                model_view_matrix: Mat4f32::identity(),
                chunk_offset: Vec3f32::zeros(),
                _padding0: Default::default(),
            },
            static_uniform_cache: StaticUniforms {
                projection_matrix: Mat4f32::identity(),
                screen_size: Vec2f32::zeros(),
                _padding0: Default::default(),
                fog_color: Vec4f32::zeros(),
                fog_range_and_game_time: Vec3f32::zeros(),
                _padding1: Default::default(),
                fog_shape: 0,
                _padding2: Default::default(),
            },
            textures: [(initial_texture, initial_sampler); 3],
        }
    }

    fn update_uniform(&mut self, data: &McUniformData) {
        match data {
            McUniformData::ModelViewMatrix(mat) => {
                if self.used_uniforms.contains(&McUniform::MODEL_VIEW_MATRIX) {
                    self.push_constant_cache.model_view_matrix = *mat;
                    self.push_constants_dirty = true;
                }
            }
            McUniformData::ProjectionMatrix(mat) => {
                if self.used_uniforms.contains(&McUniform::PROJECTION_MATRIX) {
                    self.static_uniform_cache.projection_matrix = *mat;
                    self.static_uniforms_dirty = true;
                }
            }
            McUniformData::InverseViewRotationMatrix(_) => {}
            McUniformData::TextureMatrix(_) => {}
            McUniformData::ScreenSize(size) => {
                if self.used_uniforms.contains(&McUniform::SCREEN_SIZE) {
                    self.static_uniform_cache.screen_size = *size;
                    self.static_uniforms_dirty = true;
                }
            }
            McUniformData::ColorModulator(_) => {}
            McUniformData::Light0Direction(_) => {}
            McUniformData::Light1Direction(_) => {}
            McUniformData::FogStart(start) => {
                if self.used_uniforms.contains(&McUniform::FOG_START) {
                    self.static_uniform_cache.fog_range_and_game_time[0] = *start;
                    self.static_uniforms_dirty = true;
                }
            }
            McUniformData::FogEnd(end) => {
                if self.used_uniforms.contains(&McUniform::FOG_END) {
                    self.static_uniform_cache.fog_range_and_game_time[1] = *end;
                    self.static_uniforms_dirty = true;
                }
            }
            McUniformData::FogColor(color) => {
                if self.used_uniforms.contains(&McUniform::FOG_COLOR) {
                    self.static_uniform_cache.fog_color = *color;
                    self.static_uniforms_dirty = true;
                }
            }
            McUniformData::FogShape(shape) => {
                if self.used_uniforms.contains(&McUniform::FOG_SHAPE) {
                    self.static_uniform_cache.fog_shape = *shape;
                    self.static_uniforms_dirty = true;
                }
            }
            McUniformData::LineWidth(_) => {}
            McUniformData::GameTime(time) => {
                if self.used_uniforms.contains(&McUniform::GAME_TIME) {
                    self.static_uniform_cache.fog_range_and_game_time[2] = *time;
                    self.static_uniforms_dirty = true;
                }
            }
            McUniformData::ChunkOffset(offset) => {
                if self.used_uniforms.contains(&McUniform::CHUNK_OFFSET) {
                    self.push_constant_cache.chunk_offset = *offset;
                    self.push_constants_dirty = true;
                }
            }
        }
    }

    fn update_texture(&mut self, index: u32, view: vk::ImageView, sampler: vk::Sampler) {
        match index {
            0 => {
                self.textures[0] = (view, sampler);
                self.textures_dirty = true;
            },
            1 => {
                self.textures[1] = (view, sampler);
                self.textures_dirty = true;
            },
            2 => {
                self.textures[2] = (view, sampler);
                self.textures_dirty = true;
            },
            _ => log::warn!("Called updated texture on index {:?} which is out of bounds", index),
        }
    }

    fn validate_push_constants(&mut self) -> Option<&PushConstants> {
        if self.push_constants_dirty {
            self.push_constants_dirty = false;
            Some(&self.push_constant_cache)
        } else {
            None
        }
    }

    fn validate_static_uniforms(&mut self) -> Option<&StaticUniforms> {
        if self.static_uniforms_dirty {
            self.static_uniforms_dirty = false;
            Some(&self.static_uniform_cache)
        } else {
            None
        }
    }

    fn validate_textures(&mut self) -> Option<&[(vk::ImageView, vk::Sampler); 3]> {
        if self.textures_dirty {
            self.textures_dirty = false;
            Some(&self.textures)
        } else {
            None
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct PushConstants {
    #[allow(unused)]
    model_view_matrix: Mat4f32,

    #[allow(unused)]
    chunk_offset: Vec3f32,

    _padding0: [u8; 4],
}
const_assert_eq!(std::mem::size_of::<PushConstants>(), 80);
const_assert_eq!(std::mem::size_of::<PushConstants>() % 16, 0);

unsafe impl Zeroable for PushConstants {}
unsafe impl Pod for PushConstants {}

#[repr(C)]
#[derive(Copy, Clone, Default)]
struct StaticUniforms {
    #[allow(unused)]
    projection_matrix: Mat4f32,

    #[allow(unused)]
    screen_size: Vec2f32,

    _padding0: [u8; 8],

    #[allow(unused)]
    fog_color: Vec4f32,

    #[allow(unused)]
    fog_range_and_game_time: Vec3f32,

    _padding1: [u8; 4],

    #[allow(unused)]
    fog_shape: u32,

    _padding2: [u8; 12],
}
const_assert_eq!(std::mem::size_of::<StaticUniforms>(), 128);
const_assert_eq!(std::mem::size_of::<StaticUniforms>() % 16, 0);

unsafe impl Zeroable for StaticUniforms {}
unsafe impl Pod for StaticUniforms {}

fn try_create_shader_module(device: &DeviceContext, data: &[u8], name: &str) -> Result<vk::ShaderModule, vk::Result> {
    unsafe {
        create_shader_from_bytes(device.get_functions(), data)
    }.map_err(|err| {
        log::error!("vkCreateShaderModule returned {:?} when creating module {:?}", err, name);
        err
    })
}

const SHADER_ENTRY: &'static CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") }; // GOD I LOVE RUSTS FFI API IT IS SO NICE AND DEFINITELY NOT STUPID WITH WHICH FUNCTIONS ARE CONST AND WHICH AREN'T
static DEBUG_POSITION_VERTEX_BIN: &'static [u8] = include_bytes_aligned!(4, concat!(env!("B4D_RESOURCE_DIR"), "emulator/debug_position_vert.spv"));
static DEBUG_COLOR_VERTEX_BIN: &'static [u8] = include_bytes_aligned!(4, concat!(env!("B4D_RESOURCE_DIR"), "emulator/debug_color_vert.spv"));
static DEBUG_UV_VERTEX_BIN: &'static [u8] = include_bytes_aligned!(4, concat!(env!("B4D_RESOURCE_DIR"), "emulator/debug_uv_vert.spv"));
static DEBUG_NULL_VERTEX_BIN: &'static [u8] = include_bytes_aligned!(4, concat!(env!("B4D_RESOURCE_DIR"), "emulator/debug_null_vert.spv"));
static DEBUG_FRAGMENT_BIN: &'static [u8] = include_bytes_aligned!(4, concat!(env!("B4D_RESOURCE_DIR"), "emulator/debug_frag.spv"));
static TEXTURED_FRAGMENT_BIN: &'static [u8] = include_bytes_aligned!(4, concat!(env!("B4D_RESOURCE_DIR"), "emulator/textured_frag.spv"));

static BACKGROUND_VERTEX_BIN: &'static [u8] = include_bytes_aligned!(4, concat!(env!("B4D_RESOURCE_DIR"), "emulator/background_vert.spv"));
static BACKGROUND_FRAGMENT_BIN: &'static [u8] = include_bytes_aligned!(4, concat!(env!("B4D_RESOURCE_DIR"), "emulator/background_frag.spv"));