use std::ffi::CStr;
use std::sync::{Arc, Weak};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use ash::vk;
use bumpalo::Bump;
use crate::device::device::Queue;
use crate::objects::id::BufferId;

use crate::prelude::*;
use crate::renderer::emulator::pipeline::{DrawTask, EmulatorPipeline, EmulatorPipelinePass, PipelineTask, PipelineTypeInfo, PooledObjectProvider, SubmitRecorder};
use crate::vk::objects::allocator::{Allocation, AllocationStrategy};

pub struct DepthTypeInfo {
    pub vertex_stride: u32,
    pub vertex_position_offset: u32,
    pub vertex_position_format: vk::Format,
    pub topology: vk::PrimitiveTopology,
    pub primitive_restart: bool,
    pub discard: bool,
}

pub struct DepthPipelineCore {
    device: DeviceEnvironment,
    depth_format: vk::Format,
    pipeline_layout: vk::PipelineLayout,
    render_pass: vk::RenderPass,
    pipelines: Box<[vk::Pipeline]>,
    type_infos: Box<[PipelineTypeInfo]>,
}

impl DepthPipelineCore {
    pub fn new(device: DeviceEnvironment, types: &[DepthTypeInfo]) -> Self {
        let type_infos: Box<_> = types.iter().map(|info| PipelineTypeInfo{ vertex_stride: info.vertex_stride }).collect();

        let depth_format = vk::Format::D16_UNORM;

        let pipeline_layout = Self::create_pipeline_layout(&device);
        let render_pass = Self::create_render_pass(&device, depth_format);
        let pipelines = Self::create_pipelines(&device, pipeline_layout, render_pass, types);

        Self {
            device,
            depth_format,
            pipeline_layout,
            render_pass,
            pipelines,
            type_infos
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

    fn create_pipelines(device: &DeviceEnvironment, layout: vk::PipelineLayout, render_pass: vk::RenderPass, types: &[DepthTypeInfo]) -> Box<[vk::Pipeline]> {
        let vertex_module = Self::load_shaders(device);

        let alloc = Bump::new();

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vertex_module)
                .name(DEBUG_POSITION_VERTEX_ENTRY)
                .build()
        ];

        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .scissor_count(1);

        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .line_width(1f32);

        let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(false);


        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder();

        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];

        let dynamic_state = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&dynamic_states);

        let infos: Box<_> = types.iter().map(|type_info| {
            let input_bindings = alloc.alloc([
                vk::VertexInputBindingDescription {
                    binding: 0,
                    stride: type_info.vertex_stride,
                    input_rate: vk::VertexInputRate::VERTEX
                }
            ]);

            let input_attributes = alloc.alloc([
                vk::VertexInputAttributeDescription {
                    location: 0,
                    binding: 0,
                    format: type_info.vertex_position_format,
                    offset: type_info.vertex_position_offset
                }
            ]);

            let input_state = alloc.alloc(
                vk::PipelineVertexInputStateCreateInfo::builder()
                    .vertex_binding_descriptions(input_bindings)
                    .vertex_attribute_descriptions(input_attributes)
            );

            let input_assembly_state = alloc.alloc(
                vk::PipelineInputAssemblyStateCreateInfo::builder()
                    .topology(type_info.topology)
                    .primitive_restart_enable(type_info.primitive_restart)
            );


            let depth_stencil_state = alloc.alloc(
                vk::PipelineDepthStencilStateCreateInfo::builder()
                    .depth_test_enable(true)
                    .depth_write_enable(!type_info.discard)
                    .depth_compare_op(vk::CompareOp::LESS)
            );

            vk::GraphicsPipelineCreateInfo::builder()
                .stages(&shader_stages)
                .vertex_input_state(input_state)
                .input_assembly_state(input_assembly_state)
                .viewport_state(&viewport_state)
                .rasterization_state(&rasterization_state)
                .multisample_state(&multisample_state)
                .depth_stencil_state(depth_stencil_state)
                .color_blend_state(&color_blend_state)
                .dynamic_state(&dynamic_state)
                .layout(layout)
                .render_pass(render_pass)
                .subpass(0)
                .build()
        }).collect();

        let pipelines = unsafe {
            device.vk().create_graphics_pipelines(vk::PipelineCache::null(), infos.as_ref(), None)
        }.unwrap().into_boxed_slice();

        unsafe {
            device.vk().destroy_shader_module(vertex_module, None)
        };

        pipelines
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

impl Drop for DepthPipelineCore {
    fn drop(&mut self) {
        unsafe {
            for pipeline in self.pipelines.iter() {
                self.device.vk().destroy_pipeline(*pipeline, None);
            }
            self.device.vk().destroy_render_pass(self.render_pass, None);
            self.device.vk().destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }
}

pub struct DepthPipelineConfig {
    core: Arc<DepthPipelineCore>,
    weak: Weak<DepthPipelineConfig>,
    viewport_size: Vec2u32,
    objects: Box<[DepthPipelineObjects]>,
    next_index: AtomicUsize,
    outputs: Box<[vk::ImageView]>,
}

impl DepthPipelineConfig {
    pub fn new(core: Arc<DepthPipelineCore>, viewport_size: Vec2u32) -> Arc<Self> {
        let objects: Box<[DepthPipelineObjects]> = std::iter::repeat_with(|| DepthPipelineObjects::new(&core, viewport_size)).take(2).collect();

        let outputs = objects.iter().map(|obj| obj.sampler_view).collect();

        Arc::new_cyclic(|weak| Self {
            core,
            weak: weak.clone(),
            viewport_size,
            objects,
            next_index: AtomicUsize::new(0),
            outputs,
        })
    }

    fn get_next_index(&self) -> usize {
        loop {
            let current = self.next_index.load(Ordering::SeqCst);
            let next = (current + 1) % self.objects.len();
            if self.next_index.compare_exchange(current, next, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                return current;
            }
        }
    }
}

impl EmulatorPipeline for DepthPipelineConfig {
    fn start_pass(&self) -> Box<dyn EmulatorPipelinePass + Send> {
        let index = self.get_next_index();
        self.objects[index].wait_and_take();

        Box::new(DepthPipelinePass::new(self.weak.upgrade().unwrap(), index))
    }

    fn get_type_table(&self) -> &[PipelineTypeInfo] {
        self.core.type_infos.as_ref()
    }

    fn get_outputs(&self) -> (Vec2u32, &[vk::ImageView]) {
        (self.viewport_size, self.outputs.as_ref())
    }
}

impl Drop for DepthPipelineConfig {
    fn drop(&mut self) {
        for objects in self.objects.iter_mut() {
            objects.destroy(&self.core);
        }
    }
}

struct DepthPipelineObjects {
    ready: AtomicBool,
    image: vk::Image,
    allocation: Option<Allocation>,
    framebuffer_view: vk::ImageView,
    framebuffer: vk::Framebuffer,
    sampler_view: vk::ImageView,
}

impl DepthPipelineObjects {
    fn new(core: &DepthPipelineCore, size: Vec2u32) -> Self {
        let (image, allocation) = Self::create_image(core, size);
        let (framebuffer, framebuffer_view) = Self::create_framebuffer(core, image, size);
        let sampler_view = Self::create_sampler_image(core, image);

        Self {
            ready: AtomicBool::new(true),
            image,
            allocation: Some(allocation),
            framebuffer_view,
            framebuffer,
            sampler_view
        }
    }

    fn wait_and_take(&self) {
        loop {
            if self.ready.compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                return;
            }
            std::thread::yield_now();
        }
    }

    fn destroy(&mut self, core: &DepthPipelineCore) {
        unsafe {
            core.device.vk().destroy_framebuffer(self.framebuffer, None);
            core.device.vk().destroy_image_view(self.framebuffer_view, None);
            core.device.vk().destroy_image_view(self.sampler_view, None);
            core.device.vk().destroy_image(self.image, None);
        }
        core.device.get_allocator().free(self.allocation.take().unwrap());
    }

    fn create_image(core: &DepthPipelineCore, size: Vec2u32) -> (vk::Image, Allocation) {
        let info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(core.depth_format)
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
            core.device.vk().create_image(&info, None)
        }.unwrap();

        let alloc = core.device.get_allocator().allocate_image_memory(image, &AllocationStrategy::AutoGpuOnly).unwrap();

        unsafe {
            core.device.vk().bind_image_memory(image, alloc.memory(), alloc.offset())
        }.unwrap();

        (image, alloc)
    }

    fn create_framebuffer(core: &DepthPipelineCore, image: vk::Image, size: Vec2u32) -> (vk::Framebuffer, vk::ImageView) {
        let view_info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(core.depth_format)
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY
            })
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::DEPTH,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1
            });

        let view = unsafe {
            core.device.vk().create_image_view(&view_info, None)
        }.unwrap();

        let info = vk::FramebufferCreateInfo::builder()
            .render_pass(core.render_pass)
            .attachments(std::slice::from_ref(&view))
            .width(size[0])
            .height(size[1])
            .layers(1);

        let framebuffer = unsafe {
            core.device.vk().create_framebuffer(&info, None)
        }.unwrap();

        (framebuffer, view)
    }

    fn create_sampler_image(core: &DepthPipelineCore, image: vk::Image) -> vk::ImageView {
        let view_info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(core.depth_format)
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::R,
                g: vk::ComponentSwizzle::R,
                b: vk::ComponentSwizzle::R,
                a: vk::ComponentSwizzle::ONE
            })
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::DEPTH,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1
            });

        unsafe {
            core.device.vk().create_image_view(&view_info, None)
        }.unwrap()
    }
}

struct DepthPipelinePass {
    config: Arc<DepthPipelineConfig>,
    index: usize,

    command_buffer: Option<vk::CommandBuffer>,
    current_pipeline: Option<u32>,
    current_vertex_buffer: Option<BufferId>,
    current_index_buffer: Option<BufferId>,
}

impl DepthPipelinePass {
    fn new(config: Arc<DepthPipelineConfig>, index: usize) -> Self {
        Self {
            config,
            index,

            command_buffer: None,
            current_pipeline: None,
            current_vertex_buffer: None,
            current_index_buffer: None,
        }
    }

    fn draw(&mut self, draw_task: &DrawTask) {
        let device = self.config.core.device.get_device();
        let cmd = *self.command_buffer.as_ref().unwrap();

        if self.current_pipeline != Some(draw_task.type_id) {
            let pipeline = self.config.core.pipelines[draw_task.type_id as usize];

            unsafe {
                device.vk().cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, pipeline)
            };

            self.current_pipeline = Some(draw_task.type_id);
        }

        if self.current_vertex_buffer != Some(draw_task.vertex_buffer.get_id()) {
            unsafe {
                device.vk().cmd_bind_vertex_buffers(
                    cmd,
                    0,
                    std::slice::from_ref(&draw_task.vertex_buffer.get_handle()),
                    std::slice::from_ref(&0)
                );
            }
            self.current_vertex_buffer = Some(draw_task.vertex_buffer.get_id());
        }

        if self.current_index_buffer != Some(draw_task.index_buffer.get_id()) {
            unsafe {
                device.vk().cmd_bind_index_buffer(cmd, draw_task.index_buffer.get_handle(), 0, draw_task.index_type);
            }
            self.current_index_buffer = Some(draw_task.index_buffer.get_id());
        }

        unsafe {
            device.vk().cmd_draw_indexed(cmd, draw_task.index_count, 1, draw_task.first_index, draw_task.vertex_offset, 0);
        }
    }

    fn set_model_view_matrix(&mut self, mat: Mat4f32) {
        let device = self.config.core.device.get_device();
        let cmd = *self.command_buffer.as_ref().unwrap();

        unsafe {
            device.vk().cmd_push_constants(
                cmd,
                self.config.core.pipeline_layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                crate::util::slice::to_byte_slice(std::slice::from_ref(&mat))
            )
        }
    }

    fn set_projection_matrix(&mut self, mat: Mat4f32) {
        let device = self.config.core.device.get_device();
        let cmd = *self.command_buffer.as_ref().unwrap();

        unsafe {
            device.vk().cmd_push_constants(
                cmd,
                self.config.core.pipeline_layout,
                vk::ShaderStageFlags::VERTEX,
                16 * 4,
                crate::util::slice::to_byte_slice(std::slice::from_ref(&mat))
            )
        }
    }
}

impl EmulatorPipelinePass for DepthPipelinePass {
    fn init(&mut self, _: &Queue, obj: &mut PooledObjectProvider) {
        let cmd = obj.get_begin_command_buffer().unwrap();
        self.command_buffer = Some(cmd);

        let device = self.config.core.device.get_device();

        let render_rect = vk::Rect2D {
            offset: vk::Offset2D{ x: 0, y: 0 },
            extent: vk::Extent2D{ width: self.config.viewport_size[0], height: self.config.viewport_size[1] }
        };

        let clear_values = [
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0
                }
            }
        ];
        let info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.config.core.render_pass)
            .framebuffer(self.config.objects[self.index].framebuffer)
            .render_area(render_rect)
            .clear_values(&clear_values);

        let viewports = [
            vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: self.config.viewport_size[0] as f32,
                height: self.config.viewport_size[1] as f32,
                min_depth: 0.0,
                max_depth: 1.0
            }
        ];

        unsafe {
            device.vk().cmd_begin_render_pass(cmd, &info, vk::SubpassContents::INLINE);

            device.vk().cmd_set_viewport(cmd, 0, &viewports);
            device.vk().cmd_set_scissor(cmd, 0, std::slice::from_ref(&render_rect));
        }
    }

    fn process_task(&mut self, task: PipelineTask, _: &mut PooledObjectProvider) {
        match task {
            PipelineTask::SetModelViewMatrix(mat) =>
                self.set_model_view_matrix(mat),
            PipelineTask::SetProjectionMatrix(mat) =>
                self.set_projection_matrix(mat),
            PipelineTask::Draw(draw) =>
                self.draw(&draw),
        }
    }

    fn record<'a>(&mut self, obj: &mut PooledObjectProvider, submits: &mut SubmitRecorder<'a>, alloc: &'a Bump) {
        let device = self.config.core.device.get_device();
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
            .image(self.config.objects[self.index].image)
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
}

impl Drop for DepthPipelinePass {
    fn drop(&mut self) {
        self.config.objects[self.index].ready.store(true, Ordering::SeqCst);
    }
}

const DEBUG_POSITION_VERTEX_ENTRY: &'static CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") }; // GOD I LOVE RUSTS FFI API IT IS SO NICE AND DEFINITELY NOT STUPID WITH WHICH FUNCTIONS ARE CONST AND WHICH AREN'T
const DEBUG_POSITION_VERTEX_BIN: &'static [u8] = include_bytes!(concat!(env!("B4D_RESOURCE_DIR"), "emulator/debug_position_vert.spv"));