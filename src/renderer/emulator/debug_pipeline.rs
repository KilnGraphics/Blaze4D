//! Provides a collection of [`EmulatorPipeline`] implementations useful for debugging.

use std::collections::HashMap;
use std::ffi::CStr;
use std::mem::ManuallyDrop;
use std::sync::{Arc, Mutex, Weak};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Instant;
use ash::vk;
use bumpalo::Bump;
use crate::device::device::Queue;
use crate::objects::id::BufferId;

use crate::prelude::*;
use crate::renderer::emulator::EmulatorRenderer;
use crate::renderer::emulator::mc_shaders::{DevUniform, ShaderDropListener, ShaderId, ShaderListener};
use crate::renderer::emulator::pipeline::{DrawTask, EmulatorPipeline, EmulatorPipelinePass, PipelineTask, PooledObjectProvider, SubmitRecorder};
use crate::vk::objects::allocator::{Allocation, AllocationStrategy};

pub struct DepthTypeInfo {
    pub vertex_stride: u32,
    pub vertex_position_offset: u32,
    pub vertex_position_format: vk::Format,
    pub topology: vk::PrimitiveTopology,
    pub primitive_restart: bool,
    pub discard: bool,
}

pub struct DebugPipeline {
    device: DeviceEnvironment,
    emulator: Arc<EmulatorRenderer>,
    weak: Weak<Self>,
    framebuffer_size: Vec2u32,
    vertex_module: vk::ShaderModule,
    render_pass: vk::RenderPass,
    set0_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
    pipelines: Mutex<HashMap<ShaderId, ShaderPipelines>>,
    next_index: AtomicUsize,
    // Need to control drop order
    pass_objects: ManuallyDrop<Box<[PassObjects]>>,
    output_views: Box<[vk::ImageView]>,
}
assert_impl_all!(DebugPipeline: Send, Sync);

impl DebugPipeline {
    pub fn new(device: DeviceEnvironment, emulator: Arc<EmulatorRenderer>, framebuffer_size: Vec2u32) -> Arc<Self> {
        let vertex_module = Self::load_shaders(&device);
        let render_pass = Self::create_render_pass(&device, vk::Format::D16_UNORM);
        let (set0_layout, pipeline_layout) = Self::create_pipeline_layout(&device);

        let pass_objects: Box<_> = std::iter::repeat_with(||
            PassObjects::new(&device, framebuffer_size, vk::Format::D16_UNORM, render_pass)
        ).take(2).collect();

        let output_views: Box<_> = pass_objects.iter().map(|obj| obj.depth_sampler_view).collect();

        Arc::new_cyclic(|weak| {
            Self {
                device,
                emulator,
                weak: weak.clone(),
                framebuffer_size,
                vertex_module,
                render_pass,
                set0_layout,
                pipeline_layout,
                pipelines: Mutex::new(HashMap::new()),
                next_index: AtomicUsize::new(0),
                pass_objects: ManuallyDrop::new(pass_objects),
                output_views
            }
        })
    }

    fn get_next_index(&self) -> usize {
        loop {
            let current = self.next_index.load(Ordering::SeqCst);
            let next = (current + 1) % self.pass_objects.len();
            if self.next_index.compare_exchange(current, next, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                return current;
            }
        }
    }

    fn get_pipeline(&self, shader: ShaderId, config: &PipelineConfig) -> vk::Pipeline {
        let mut guard = self.pipelines.lock().unwrap();
        let pipelines = guard.get_mut(&shader).unwrap_or_else(|| {
            log::error!("Called get_pipeline for unregistered shader {:?}", shader);
            panic!()
        });

        pipelines.get_or_create_pipeline(config, || self.create_pipeline(config, shader))
    }

    fn create_pipeline(&self, config: &PipelineConfig, shader: ShaderId) -> vk::Pipeline {
        let shader = self.emulator.get_shader(shader).unwrap_or_else(|| {
            log::error!("Unable to find shader {:?} in DebugPipeline::create_pipeline", shader);
            panic!();
        });
        let vertex_format = shader.get_vertex_format();

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(self.vertex_module)
                .name(DEBUG_POSITION_VERTEX_ENTRY)
                .build()
        ];

        let input_bindings = [
            vk::VertexInputBindingDescription {
                binding: 0,
                stride: vertex_format.stride,
                input_rate: vk::VertexInputRate::VERTEX
            }
        ];

        let input_attributes = [
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: vertex_format.position.format,
                offset: vertex_format.position.offset,
            }
        ];

        let input_state = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&input_bindings)
            .vertex_attribute_descriptions(&input_attributes);

        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: self.framebuffer_size[0] as f32,
            height: self.framebuffer_size[1] as f32,
            min_depth: 0.0,
            max_depth: 1.0
        };

        let scissor = vk::Rect2D {
            offset: vk::Offset2D{ x: 0, y: 0 },
            extent: vk::Extent2D{ width: self.framebuffer_size[0], height: self.framebuffer_size[1] }
        };

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


        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder();

        let dynamic_state = vk::PipelineDynamicStateCreateInfo::builder();

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(config.primitive_topology)
            .primitive_restart_enable(true);

        let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS);

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
            .layout(self.pipeline_layout)
            .render_pass(self.render_pass)
            .subpass(0);

        let pipeline = *unsafe {
            self.device.vk().create_graphics_pipelines(vk::PipelineCache::null(), std::slice::from_ref(&info), None)
        }.unwrap().get(0).unwrap();

        pipeline
    }

    fn create_pipeline_layout(device: &DeviceEnvironment) -> (vk::DescriptorSetLayout, vk::PipelineLayout) {
        let bindings = [
            vk::DescriptorSetLayoutBinding {
                binding: 0,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::ALL,
                p_immutable_samplers: std::ptr::null(),
            }
        ];

        let info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&bindings);

        let set_layout = unsafe {
            device.vk().create_descriptor_set_layout(&info, None)
        }.unwrap();

        let info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(std::slice::from_ref(&set_layout));

        let pipeline_layout = unsafe {
            device.vk().create_pipeline_layout(&info, None)
        }.unwrap();

        (set_layout, pipeline_layout)
    }

    fn create_render_pass(device: &DeviceEnvironment, depth_format: vk::Format) -> vk::RenderPass {
        let attachments = [
            vk::AttachmentDescription::builder()
                .format(depth_format)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .build()
        ];

        let depth_ref = vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .depth_stencil_attachment(&depth_ref);

        let info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachments)
            .subpasses(std::slice::from_ref(&subpass));

        unsafe {
            device.vk().create_render_pass(&info, None)
        }.unwrap()
    }

    fn load_shaders(device: &DeviceEnvironment) -> vk::ShaderModule {
        let info = vk::ShaderModuleCreateInfo::builder()
            .code(crate::util::slice::from_byte_slice(DEBUG_POSITION_VERTEX_BIN));

        let vertex = unsafe {
            device.vk().create_shader_module(&info, None)
        }.unwrap();

        vertex
    }
}

impl EmulatorPipeline for DebugPipeline {
    fn start_pass(&self) -> Box<dyn EmulatorPipelinePass + Send> {
        let index = self.get_next_index();
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
            let mut  pipelines = ShaderPipelines::new(&self.device, shader, listener);
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
        for objects in self.pass_objects.iter_mut() {
            objects.destroy(&self.device);
        }
        unsafe {
            ManuallyDrop::drop(&mut self.pass_objects);
        }
        self.pipelines.get_mut().unwrap().clear();
        unsafe {
            self.device.vk().destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.vk().destroy_descriptor_set_layout(self.set0_layout, None);
            self.device.vk().destroy_render_pass(self.render_pass, None);
            self.device.vk().destroy_shader_module(self.vertex_module, None);
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct PipelineConfig {
    primitive_topology: vk::PrimitiveTopology,
}

struct ShaderPipelines {
    device: Arc<DeviceContext>,
    shader: ShaderId,
    pipelines: HashMap<PipelineConfig, vk::Pipeline>,
    #[allow(unused)]
    listener: ShaderListener,
    used_counter: u32,
    marked: bool,
}

impl ShaderPipelines {
    fn new(device: &DeviceEnvironment, shader: ShaderId, listener: ShaderListener) -> Self {
        Self {
            device: device.get_device().clone(),
            shader,
            pipelines: HashMap::new(),
            listener,
            used_counter: 0,
            marked: false,
        }
    }

    fn get_or_create_pipeline<T: FnOnce() -> vk::Pipeline>(&mut self, config: &PipelineConfig, create_fn: T) -> vk::Pipeline {
        if let Some(pipeline) = self.pipelines.get(config) {
            *pipeline
        } else {
            let pipeline = create_fn();
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

struct PassObjects {
    ready: AtomicBool,

    depth_image: vk::Image,
    depth_allocation: Option<Allocation>,
    depth_framebuffer_view: vk::ImageView,
    depth_sampler_view: vk::ImageView,

    framebuffer: vk::Framebuffer,
}

impl PassObjects {
    fn new(device: &DeviceEnvironment, framebuffer_size: Vec2u32, depth_format: vk::Format, render_pass: vk::RenderPass) -> Self {
        let (depth_image, depth_allocation, depth_framebuffer_view, depth_sampler_view) = Self::create_depth_image(device, framebuffer_size, depth_format);
        let framebuffer = Self::create_framebuffer(device, framebuffer_size, depth_framebuffer_view, render_pass);

        Self {
            ready: AtomicBool::new(true),
            depth_image,
            depth_allocation: Some(depth_allocation),
            depth_framebuffer_view,
            depth_sampler_view,
            framebuffer
        }
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

    fn destroy(&mut self, device: &DeviceEnvironment) {
        unsafe {
            device.vk().destroy_framebuffer(self.framebuffer, None);
            device.vk().destroy_image_view(self.depth_sampler_view, None);
            device.vk().destroy_image_view(self.depth_framebuffer_view, None);
            device.vk().destroy_image(self.depth_image, None);
        }
        device.get_allocator().free(self.depth_allocation.take().unwrap());
    }

    fn create_depth_image(device: &DeviceEnvironment, size: Vec2u32, format: vk::Format) -> (vk::Image, Allocation, vk::ImageView, vk::ImageView) {
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
            .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::SAMPLED)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED);

        let image = unsafe {
            device.vk().create_image(&info, None)
        }.unwrap();

        let alloc = device.get_allocator().allocate_image_memory(image, &AllocationStrategy::AutoGpuOnly).unwrap();

        unsafe {
            device.vk().bind_image_memory(image, alloc.memory(), alloc.offset())
        }.unwrap();

        let info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            })
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::DEPTH,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1
            });

        let framebuffer_view = unsafe {
            device.vk().create_image_view(&info, None)
        }.unwrap();

        let info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::R,
                g: vk::ComponentSwizzle::R,
                b: vk::ComponentSwizzle::R,
                a: vk::ComponentSwizzle::ONE,
            })
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::DEPTH,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1
            });

        let sampler_view = unsafe {
            device.vk().create_image_view(&info, None)
        }.unwrap();

        (image, alloc, framebuffer_view, sampler_view)
    }

    fn create_framebuffer(device: &DeviceEnvironment, size: Vec2u32, depth_view: vk::ImageView, redner_pass: vk::RenderPass) -> vk::Framebuffer {
        let info = vk::FramebufferCreateInfo::builder()
            .render_pass(redner_pass)
            .attachments(std::slice::from_ref(&depth_view))
            .width(size[0])
            .height(size[1])
            .layers(1);

        unsafe {
            device.vk().create_framebuffer(&info, None)
        }.unwrap()
    }
}

struct DebugPipelinePass {
    parent: Arc<DebugPipeline>,
    index: usize,

    shader_uniforms: HashMap<ShaderId, (vk::Buffer, vk::DeviceSize)>,

    command_buffer: Option<vk::CommandBuffer>,
    current_pipeline: Option<(ShaderId, PipelineConfig)>,
    current_vertex_buffer: Option<BufferId>,
    current_index_buffer: Option<BufferId>,
}

impl DebugPipelinePass {
    fn new(parent: Arc<DebugPipeline>, index: usize) -> Self {
        Self {
            parent,
            index,

            shader_uniforms: HashMap::new(),

            command_buffer: None,
            current_pipeline: None,
            current_vertex_buffer: None,
            current_index_buffer: None
        }
    }

    fn update_dev_uniform(&mut self, shader: ShaderId, buffer: vk::Buffer, offset: vk::DeviceSize) {
        self.shader_uniforms.insert(shader, (buffer, offset));
        if let Some(current) = &self.current_pipeline {
            if current.0 == shader {
                self.push_dev_uniforms(buffer, offset);
            }
        }
    }

    fn draw(&mut self, task: &DrawTask) {
        let device = self.parent.device.get_device();
        let cmd = *self.command_buffer.as_ref().unwrap();

        let pipeline_config = PipelineConfig {
            primitive_topology: task.primitive_topology
        };

        if self.current_pipeline != Some((task.shader, pipeline_config)) {
            self.current_pipeline = Some((task.shader, pipeline_config));

            let new_pipeline = self.parent.get_pipeline(task.shader, &pipeline_config);
            unsafe {
                device.vk().cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, new_pipeline);
            }

            if let Some((buffer, offset)) = self.shader_uniforms.get(&task.shader) {
                self.push_dev_uniforms(*buffer, *offset);
            } else {
                log::warn!("Called draw with no uniform data. skipping!");
                return;
            }
        }

        if self.current_vertex_buffer != Some(task.vertex_buffer.get_id()) {
            unsafe {
                device.vk().cmd_bind_vertex_buffers(
                    cmd,
                    0,
                    std::slice::from_ref(&task.vertex_buffer.get_handle()),
                    std::slice::from_ref(&0)
                );
            }
            self.current_vertex_buffer = Some(task.vertex_buffer.get_id());
        }

        if self.current_index_buffer != Some(task.index_buffer.get_id()) {
            unsafe {
                device.vk().cmd_bind_index_buffer(cmd, task.index_buffer.get_handle(), 0, task.index_type);
            }
            self.current_index_buffer = Some(task.index_buffer.get_id());
        }

        unsafe {
            device.vk().cmd_draw_indexed(cmd, task.index_count, 1, task.first_index, task.vertex_offset, 0);
        }
    }

    fn push_dev_uniforms(&self, buffer: vk::Buffer, offset: vk::DeviceSize) {
        let buffer_info = vk::DescriptorBufferInfo {
            buffer,
            offset,
            range: std::mem::size_of::<DevUniform>() as vk::DeviceSize,
        };
        let write = vk::WriteDescriptorSet::builder()
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(std::slice::from_ref(&buffer_info));

        unsafe {
            self.parent.device.get_device().push_descriptor_khr().cmd_push_descriptor_set(
                self.command_buffer.unwrap(),
                vk::PipelineBindPoint::GRAPHICS,
                self.parent.pipeline_layout,
                0,
                std::slice::from_ref(&write)
            );
        }
    }
}

impl EmulatorPipelinePass for DebugPipelinePass {
    fn init(&mut self, _: &Queue, obj: &mut PooledObjectProvider) {
        let cmd = obj.get_begin_command_buffer().unwrap();
        self.command_buffer = Some(cmd);

        let device = self.parent.device.get_device();

        let clear_values = [
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0
                }
            }
        ];
        let info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.parent.render_pass)
            .framebuffer(self.parent.pass_objects[self.index].framebuffer)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D{ x: 0, y: 0 },
                extent: vk::Extent2D{ width: self.parent.framebuffer_size[0], height: self.parent.framebuffer_size[1] }
            })
            .clear_values(&clear_values);

        unsafe {
            device.vk().cmd_begin_render_pass(cmd, &info, vk::SubpassContents::INLINE);
        }
    }

    fn process_task(&mut self, task: &PipelineTask, _: &mut PooledObjectProvider) {
        match task {
            PipelineTask::UpdateDevUniform(shader, buffer, offset) => {
                self.update_dev_uniform(*shader, *buffer, *offset);
            }
            PipelineTask::Draw(task) => {
                self.draw(task);
            }
        }
    }

    fn record<'a>(&mut self, _: &mut PooledObjectProvider, submits: &mut SubmitRecorder<'a>, alloc: &'a Bump) {
        let device = self.parent.device.get_device();
        let cmd = self.command_buffer.take().unwrap();

        let image_barrier = vk::ImageMemoryBarrier2::builder()
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
            });

        let info = vk::DependencyInfo::builder()
            .image_memory_barriers(std::slice::from_ref(&image_barrier));

        unsafe {
            device.vk().cmd_end_render_pass(cmd);

            device.vk().cmd_pipeline_barrier2(cmd, &info);

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

const DEBUG_POSITION_VERTEX_ENTRY: &'static CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") }; // GOD I LOVE RUSTS FFI API IT IS SO NICE AND DEFINITELY NOT STUPID WITH WHICH FUNCTIONS ARE CONST AND WHICH AREN'T
const DEBUG_POSITION_VERTEX_BIN: &'static [u8] = include_bytes!(concat!(env!("B4D_RESOURCE_DIR"), "emulator/debug_position_vert.spv"));