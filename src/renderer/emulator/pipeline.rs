use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::iter::Map;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64};
use std::sync::Mutex;
use ash::vk;
use winit::event::VirtualKeyCode::Mute;
use crate::vk::DeviceEnvironment;

#[derive(Copy, Clone)]
pub struct VertexFormat {
    vertex_size: u32,
    position: VertexFormatEntry,
    normal: Option<VertexFormatEntry>,
    color: Option<VertexFormatEntry>,
    uv: Option<VertexFormatEntry>,
}

#[derive(Copy, Clone)]
pub struct VertexFormatEntry {
    format: vk::Format,
    offset: u32,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct PipelineId(u32);

impl PipelineId {
    pub fn from_raw(id: u32) -> Self {
        Self(id)
    }

    pub fn get_raw(&self) -> u32 {
        self.0
    }
}

struct PipelineSync {
    pipelines: HashMap<PipelineId, Pipeline>,
    next_id: u32,
}

pub(super) struct PipelineManager {
    device: DeviceEnvironment,
    sync: Mutex<PipelineSync>,
    set_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
    render_pass: vk::RenderPass,
}

impl PipelineManager {
    pub(super) fn new(device: DeviceEnvironment) -> Self {
        let binding = vk::DescriptorSetLayoutBinding::builder()
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX);

        let info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(std::slice::from_ref(&binding));

        let set_layout = unsafe {
            device.vk().create_descriptor_set_layout(&info, None)
        }.unwrap();

        let constants = vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::VERTEX,
            offset: 0,
            size: 128,
        };

        let info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(std::slice::from_ref(&set_layout))
            .push_constant_ranges(std::slice::from_ref(&constants));

        let pipeline_layout = unsafe {
            device.vk().create_pipeline_layout(&info, None)
        }.unwrap();

        let attachments = [
            vk::AttachmentDescription::builder()
                .format(vk::Format::R8G8B8A8_SRGB)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
                .build(),
            vk::AttachmentDescription::builder()
                .format(vk::Format::D24_UNORM_S8_UINT)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .build()
        ];

        let color_attachment = vk::AttachmentReference::builder()
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .attachment(0);

        let depth_attachment = vk::AttachmentReference::builder()
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .attachment(1);

        let subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(std::slice::from_ref(&color_attachment))
            .depth_stencil_attachment(&depth_attachment);

        let info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachments)
            .subpasses(std::slice::from_ref(&subpass));

        let render_pass = unsafe {
            device.vk().create_render_pass(&info, None)
        }.unwrap();

        Self {
            device,
            sync: Mutex::new(PipelineSync {
                pipelines: HashMap::new(),
                next_id: 1
            }),
            set_layout,
            pipeline_layout,
            render_pass,
        }
    }

    pub(super) fn get_render_pass(&self) -> vk::RenderPass {
        self.render_pass
    }

    // TODO more very temporary (and unsafe since we dont handle errors well)
    pub fn create_default_pipeline(&mut self, vertex_format: VertexFormat) -> PipelineId {
        let vertex_code = crate::util::slice::from_byte_slice(BASIC_VERTEX_SHADER);
        let info = vk::ShaderModuleCreateInfo::builder()
            .code(vertex_code);

        let vertex_module = unsafe {
            self.device.vk().create_shader_module(&info, None)
        }.unwrap();

        let fragment_code = crate::util::slice::from_byte_slice(BASIC_FRAGMENT_SHADER);
        let info = vk::ShaderModuleCreateInfo::builder()
            .code(fragment_code);

        let fragment_module = unsafe {
            self.device.vk().create_shader_module(&info, None)
        }.unwrap();

        let entry = CString::from(CStr::from_bytes_with_nul(b"main\0").unwrap());

        let info = PipelineCreateInfo {
            vertex_shader: vertex_module,
            vertex_entry: entry.clone(),
            fragment_shader: fragment_module,
            fragment_entry: entry,
            vertex_format
        };

        let id = self.create_pipeline(&info);

        unsafe {
            self.device.vk().destroy_shader_module(vertex_module, None);
            self.device.vk().destroy_shader_module(fragment_module, None);
        }

        id
    }

    // TODO this is very temporary
    pub fn create_pipeline(&mut self, info: &PipelineCreateInfo) -> PipelineId {
        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(info.vertex_shader)
                .name(info.vertex_entry.as_c_str())
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(info.fragment_shader)
                .name(info.fragment_entry.as_c_str())
                .build()
        ];

        let vertex_binding = vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(info.vertex_format.vertex_size)
            .input_rate(vk::VertexInputRate::VERTEX);

        let vertex_attribute = vk::VertexInputAttributeDescription::builder()
            .location(0)
            .binding(0)
            .format(info.vertex_format.position.format)
            .offset(info.vertex_format.position.offset);

        let input_state = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(std::slice::from_ref(&vertex_binding))
            .vertex_attribute_descriptions(std::slice::from_ref(&vertex_attribute));

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewport = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .scissor_count(1);

        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE);

        let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::GREATER_OR_EQUAL);

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

        let pinfo = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&input_state)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport)
            .rasterization_state(&rasterization_state)
            .depth_stencil_state(&depth_stencil_state)
            .color_blend_state(&color_blend)
            .dynamic_state(&dynamic_state)
            .layout(self.pipeline_layout)
            .render_pass(self.render_pass)
            .subpass(0);

        let pipeline = * unsafe {
            self.device.vk().create_graphics_pipelines(vk::PipelineCache::null(),std::slice::from_ref(&pinfo), None)
        }.unwrap().get(0).unwrap();

        let mut guard = self.sync.lock().unwrap();
        let id = guard.next_id;
        guard.next_id += 1;

        guard.pipelines.insert(PipelineId(id), Pipeline {
            drop_after_frame: None,
            pipeline,
            vertex_format: info.vertex_format,
        });

        PipelineId(id)
    }
}

pub struct Pipeline {
    drop_after_frame: Option<u64>,
    pipeline: vk::Pipeline,
    vertex_format: VertexFormat,
}

// TODO this is very temporary
pub struct PipelineCreateInfo {
    vertex_shader: vk::ShaderModule,
    vertex_entry: CString,
    fragment_shader: vk::ShaderModule,
    fragment_entry: CString,
    vertex_format: VertexFormat,
}

const BASIC_VERTEX_SHADER: &'static [u8] = include_bytes!(concat!(env!("B4D_RESOURCE_DIR"), "debug/basic_vert.spv"));
const BASIC_FRAGMENT_SHADER: &'static [u8] = include_bytes!(concat!(env!("B4D_RESOURCE_DIR"), "debug/basic_frag.spv"));