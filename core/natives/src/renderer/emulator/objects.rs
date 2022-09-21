use std::cell::UnsafeCell;
use std::cmp::Ordering;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::ffi::CString;
use std::hash::{BuildHasher, Hash, Hasher};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use ash::vk;
use bumpalo::Bump;
use ordered_float::NotNan;
use crate::allocator::{Allocation, HostAccess};

use super::share::Share2;
use crate::define_uuid_type;

use crate::prelude::*;

macro_rules! id_type {
    ($name: ident, $id_func: expr) => {
        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                $id_func(self).eq(&$id_func(other))
            }
        }

        impl Eq for $name {
        }

        impl PartialOrd for $name {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                $id_func(self).partial_cmp(&$id_func(other))
            }
        }

        impl Ord for $name {
            fn cmp(&self, other: &Self) -> Ordering {
                $id_func(self).cmp(&$id_func(other))
            }
        }

        impl Hash for $name {
            fn hash<H: Hasher>(&self, state: &mut H) {
                $id_func(self).hash(state)
            }
        }
    };
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct BufferInfo {
    pub size: vk::DeviceSize,
}

define_uuid_type!(pub, BufferId);

pub struct Buffer {
    share: Arc<Share2>,
    id: BufferId,
    info: BufferInfo,
    handle: vk::Buffer,
    alloc_type: BufferAllocationType,
}

impl Buffer {
    pub(super) fn new_persistent(share: Arc<Share2>, size: vk::DeviceSize) -> Self {
        let id = BufferId::new();

        let info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(vk::BufferUsageFlags::TRANSFER_SRC | vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::INDEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let (handle, allocation, _) = unsafe {
            share.get_device().get_allocator().create_buffer(&info, HostAccess::None, &format_args!(""))
        }.expect("Failed to create persistent buffer");

        Self {
            share,
            id,
            info: BufferInfo {
                size
            },
            handle,
            alloc_type: BufferAllocationType::Persistent(allocation),
        }
    }

    pub fn get_id(&self) -> BufferId {
        self.id
    }

    pub fn get_info(&self) -> &BufferInfo {
        &self.info
    }

    pub(super) fn get_handle(&self) -> vk::Buffer {
        self.handle
    }

    pub(super) fn get_offset(&self) -> vk::DeviceSize {
        0
    }
}

id_type!(Buffer, Buffer::get_id);

impl Drop for Buffer {
    fn drop(&mut self) {
        match self.alloc_type {
            BufferAllocationType::Persistent(allocation) => unsafe {
                self.share.get_device().get_allocator().destroy_buffer(self.handle, allocation)
            },
        }
    }
}

pub(super) enum BufferAllocationType {
    Persistent(Allocation),
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ImageSize {
    Type1D {
        size: u32,
        mip_levels: u32,
        array_layers: u32,
    },
    Type2D {
        size: Vec2u32,
        mip_levels: u32,
        array_layers: u32,
    },
    Type3D {
        size: Vec3u32,
        mip_levels: u32,
    },
}

impl ImageSize {
    pub fn new_1d(size: u32, mip_levels: u32, array_layers: u32) -> Self {
        Self::Type1D {
            size,
            mip_levels,
            array_layers
        }
    }

    pub fn new_2d(size: Vec2u32, mip_levels: u32, array_layers: u32) -> Self {
        Self::Type2D {
            size,
            mip_levels,
            array_layers
        }
    }

    pub fn new_3d(size: Vec3u32, mip_levels: u32) -> Self {
        Self::Type3D {
            size,
            mip_levels
        }
    }

    pub fn get_size_as_vec3(&self) -> Vec3u32 {
        match self {
            ImageSize::Type1D { size, .. } => Vec3u32::new(*size, 1, 1),
            ImageSize::Type2D { size, .. } => Vec3u32::new(size[0], size[1], 1),
            ImageSize::Type3D { size, .. } => *size,
        }
    }

    pub fn get_vk_extent3d(&self) -> vk::Extent3D {
        let size = self.get_size_as_vec3();
        vk::Extent3D {
            width: size[0],
            height: size[1],
            depth: size[2],
        }
    }

    pub fn get_mip_levels(&self) -> u32 {
        match self {
            ImageSize::Type1D { mip_levels, .. } |
            ImageSize::Type2D { mip_levels, .. } |
            ImageSize::Type3D { mip_levels, .. } => *mip_levels,
        }
    }

    pub fn get_array_layers(&self) -> u32 {
        match self {
            ImageSize::Type1D { array_layers, .. } |
            ImageSize::Type2D { array_layers, .. } => *array_layers,
            ImageSize::Type3D { .. } => 1,
        }
    }

    pub fn is_1d(&self) -> bool {
        match self {
            ImageSize::Type1D { .. } => true,
            _ => false,
        }
    }

    pub fn is_2d(&self) -> bool {
        match self {
            ImageSize::Type2D { .. } => true,
            _ => false,
        }
    }

    pub fn is_3d(&self) -> bool {
        match self {
            ImageSize::Type3D { .. } => true,
            _ => false,
        }
    }

    pub fn get_vk_image_type(&self) -> vk::ImageType {
        match self {
            ImageSize::Type1D { .. } => vk::ImageType::TYPE_1D,
            ImageSize::Type2D { .. } => vk::ImageType::TYPE_2D,
            ImageSize::Type3D { .. } => vk::ImageType::TYPE_3D,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ImageInfo {
    pub size: ImageSize,
    pub format: vk::Format,
    pub aspect_mask: vk::ImageAspectFlags,
}

impl ImageInfo {
    pub fn get_full_subresource_range(&self) -> vk::ImageSubresourceRange {
        vk::ImageSubresourceRange {
            aspect_mask: self.aspect_mask,
            base_mip_level: 0,
            level_count: vk::REMAINING_MIP_LEVELS,
            base_array_layer: 0,
            layer_count: vk::REMAINING_ARRAY_LAYERS
        }
    }

    pub fn get_full_subresource_layers(&self, mip_level: u32) -> vk::ImageSubresourceLayers {
        vk::ImageSubresourceLayers {
            aspect_mask: self.aspect_mask,
            mip_level,
            base_array_layer: 0,
            layer_count: self.size.get_array_layers()
        }
    }
}

define_uuid_type!(pub, ImageId);

pub struct Image {
    share: Arc<Share2>,
    id: ImageId,
    info: ImageInfo,
    handle: vk::Image,
    allocation: Allocation,
    view: vk::ImageView,
    // mutable
    current_layout: UnsafeCell<vk::ImageLayout>,
}

impl Image {
    pub(super) fn new_persistent_color(share: Arc<Share2>, format: vk::Format, size: ImageSize) -> Self {
        let id = ImageId::new();

        let info = vk::ImageCreateInfo::builder()
            .image_type(size.get_vk_image_type())
            .format(format)
            .extent(size.get_vk_extent3d())
            .mip_levels(size.get_mip_levels())
            .array_layers(size.get_array_layers())
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let (handle, allocation, _) = unsafe {
            share.get_device().get_allocator().create_image(&info, HostAccess::None, &format_args!(""))
        }.expect("Failed to create persistent image");

        let image_info = ImageInfo {
            size,
            format,
            aspect_mask: vk::ImageAspectFlags::COLOR,
        };

        let info = vk::ImageViewCreateInfo::builder()
            .image(handle)
            .view_type(Self::get_base_image_view_type(&size))
            .format(image_info.format)
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY
            })
            .subresource_range(image_info.get_full_subresource_range());

        let view = unsafe {
            share.get_device().vk().create_image_view(&info, None)
        }.map_err(|err| {
            unsafe { share.get_device().get_allocator().destroy_image(handle, allocation) };
            err
        }).expect("Failed to create image view for persistent image");

        Self {
            share,
            id,
            info: image_info,
            handle,
            allocation,
            view,
            current_layout: UnsafeCell::new(vk::ImageLayout::UNDEFINED),
        }
    }

    pub fn get_id(&self) -> ImageId {
        self.id
    }

    pub fn get_info(&self) -> &ImageInfo {
        &self.info
    }

    pub(super) fn get_handle(&self) -> vk::Image {
        self.handle
    }

    pub(super) fn get_default_view_handle(&self) -> vk::ImageView {
        self.view
    }

    /// # Safety
    /// Must only be called from the emulator worker thread. **No memory barrier or other sync is
    /// performed**.
    pub(super) unsafe fn get_current_layout(&self) -> vk::ImageLayout {
        *self.current_layout.get().as_ref().unwrap_unchecked()
    }

    /// # Safety
    /// Must only be called from the emulator worker thread. **No memory barrier or other sync is
    /// performed**.
    pub(super) unsafe fn set_current_layout(&self, layout: vk::ImageLayout) {
        *self.current_layout.get().as_mut().unwrap_unchecked() = layout;
    }

    fn get_base_image_view_type(size: &ImageSize) -> vk::ImageViewType {
        match (size.get_vk_image_type(), size.get_array_layers()) {
            (vk::ImageType::TYPE_1D, 1) => vk::ImageViewType::TYPE_1D,
            (vk::ImageType::TYPE_1D, _) => vk::ImageViewType::TYPE_1D_ARRAY,
            (vk::ImageType::TYPE_2D, 1) => vk::ImageViewType::TYPE_2D,
            (vk::ImageType::TYPE_2D, _) => vk::ImageViewType::TYPE_2D_ARRAY,
            (vk::ImageType::TYPE_3D, _) => vk::ImageViewType::TYPE_3D,
            _ => panic!("Invalid image type"),
        }
    }
}

id_type!(Image, Image::get_id);

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            self.share.get_device().vk().destroy_image_view(self.view, None);
            self.share.get_device().get_allocator().destroy_image(self.handle, self.allocation);
        }
    }
}

// Needed because of UnsafeCell
unsafe impl Send for Image {
}
unsafe impl Sync for Image {
}

define_uuid_type!(pub, GraphicsPipelineId);

pub struct GraphicsPipeline {
    share: Arc<Share2>,
    id: GraphicsPipelineId,
}

impl GraphicsPipeline {
    pub fn get_id(&self) -> GraphicsPipelineId {
        self.id
    }

    pub(super) fn get_input_attribute_count(&self) -> u32 {
        todo!()
    }

    pub(super) fn get_color_attachment_count(&self) -> u32 {
        todo!()
    }

    pub(super) fn get_shader_stages(&self) -> &[ShaderStageInfo] {
        todo!()
    }

    pub(super) fn get_pipeline_layout(&self) -> vk::PipelineLayout {
        todo!()
    }
}

struct PipelineInstanceCache {
    device: Arc<DeviceContext>,
    instances: HashMap<u64, vk::Pipeline>,
}

impl PipelineInstanceCache {
    fn new(device: Arc<DeviceContext>) -> Self {
        Self {
            device,
            instances: HashMap::new()
        }
    }

    fn get_or_create_instance(&mut self, bump: &Bump, shader_stages: &[ShaderStageInfo], layout: vk::PipelineLayout, state: &PipelineStaticState, hasher: &mut RandomState) -> (u64, vk::Pipeline) {
        let mut hasher = hasher.build_hasher();
        state.hash(&mut hasher);
        let hash = hasher.finish();

        if let Some(handle) = self.instances.get(&hash) {
            (hash, *handle)
        } else {
            let instance = self.create_instance(bump, shader_stages, layout, state);
            self.instances.insert(hash, instance);

            (hash, instance)
        }
    }

    fn create_instance(&self, bump: &Bump, shader_stages: &[ShaderStageInfo], layout: vk::PipelineLayout, state: &PipelineStaticState) -> vk::Pipeline {
        let mut dynamic_state = Vec::with_capacity(32);
        dynamic_state.push(vk::DynamicState::VIEWPORT);
        dynamic_state.push(vk::DynamicState::SCISSOR);

        let shader_stages = bump.alloc_slice_fill_iter(shader_stages.iter().map(|stage| {
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(stage.stage)
                .module(stage.module)
                .name(&stage.entry)
                .build()
        }));

        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .scissor_count(1);

        let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let mut info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(shader_stages)
            .vertex_input_state(state.generate_vertex_input_state(bump, &mut dynamic_state))
            .input_assembly_state(state.generate_input_assembly_state(bump, &mut dynamic_state))
            .viewport_state(&viewport_state)
            .rasterization_state(state.generate_rasterization_state(bump, &mut dynamic_state))
            .multisample_state(&multisample_state)
            .depth_stencil_state(state.generate_depth_stencil_state(bump, &mut dynamic_state))
            .color_blend_state(state.generate_color_blend_state(bump, &mut dynamic_state));

        let dynamic_state = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&dynamic_state);

        info = info.dynamic_state(&dynamic_state)
            .layout(layout);
        if let Some((render_pass, subpass)) = &state.render_pass {
            info = info.render_pass(*render_pass)
                .subpass(*subpass);
        }

        *unsafe {
            self.device.vk().create_graphics_pipelines(vk::PipelineCache::null(), std::slice::from_ref(&info), None)
        }.expect("Failed to create emulator graphics pipeline instance").first().unwrap()
    }
}

impl Drop for PipelineInstanceCache {
    fn drop(&mut self) {
        for (_, handle) in &self.instances {
            unsafe {
                self.device.vk().destroy_pipeline(*handle, None);
            }
        }
    }
}

struct DynamicStateCapabilities {
    extended_dynamic_state: bool,
    extended_dynamic_state_2: bool,
}

struct PipelineDynamicState {

}

/// Contains all pipeline state needed to create a pipeline instance. Any state which may be dynamic
/// is wrapped in an Option which if set to [`None`] indicates that the state is dynamic.
#[derive(Clone, PartialEq, Hash, Debug)]
struct PipelineStaticState<'a> {
    /// (location, format, input_rate), index is the binding index
    input_attributes: &'a [(u32, vk::Format, vk::VertexInputRate)],
    /// Set to [`None`] if `VK_DYNAMIC_STATE_VERTEX_INPUT_BINDING_STRIDE` is enabled.
    input_binding_strides: Option<&'a [u32]>,
    /// Set to [`None`] if `VK_DYNAMIC_STATE_PRIMITIVE_TOPOLOGY` is enabled.
    primitive_topology: Option<vk::PrimitiveTopology>,
    /// Set to [`None`] if `VK_DYNAMIC_STATE_PRIMITIVE_RESTART_ENABLE` is enabled.
    primitive_restart_enable: Option<bool>,
    depth_clamp_enable: bool,
    rasterizer_discard_enable: bool,
    polygon_mode: vk::PolygonMode,
    /// Set to [`None`] if `VK_DYNAMIC_STATE_CULL_MODE` is enabled.
    cull_mode: Option<vk::CullModeFlags>,
    /// Set to [`None`] if `VK_DYNAMIC_STATE_FRONT_FACE` is enabled.
    front_face: Option<vk::FrontFace>,
    /// Set to [`None`] if `VK_DYNAMIC_STATE_LINE_WIDTH` is enabled.
    line_width: Option<NotNan<f32>>,
    /// Set to [`None`] if `VK_DYNAMIC_STATE_DEPTH_TEST_ENABLE` is enabled.
    depth_test_enable: Option<bool>,
    /// Set to [`None`] if `VK_DYNAMIC_STATE_DEPTH_WRITE_ENABLE` is enabled.
    depth_write_enable: Option<bool>,
    /// Set to [`None`] if `VK_DYNAMIC_STATE_DEPTH_COMPARE_OP` is enabled.
    depth_compare_op: Option<vk::CompareOp>,
    /// Set to [`None`] if `VK_DYNAMIC_STATE_STENCIL_TEST_ENABLE` is enabled.
    stencil_test_enable: Option<bool>,
    /// Set to [`None`] if `VK_DYNAMIC_STATE_STENCIL_OP` is enabled.
    /// The first element is for the front test and the second for the back test.
    stencil_op: Option<(PipelineDynamicStateStencilOp, PipelineDynamicStateStencilOp)>,
    /// Set to [`None`] if `VK_DYNAMIC_STATE_STENCIL_COMPARE_MASK` is enabled.
    /// The first element is for the front test and the second for the back test.
    stencil_compare_mask: Option<(u32, u32)>,
    /// Set to [`None`] if `VK_DYNAMIC_STATE_STENCIL_WRITE_MASK` is enabled.
    /// The first element is for the front test and the second for the back test.
    stencil_write_mask: Option<(u32, u32)>,
    /// Set to [`None`] if `VK_DYNAMIC_STATE_STENCIL_REFERENCE` is enabled.
    /// The first element is for the front test and the second for the back test.
    stencil_reference: Option<(u32, u32)>,
    /// The used logic op. [`None`] indicates that logic op is disabled. This cannot be set
    /// dynamically.
    logic_op: Option<vk::LogicOp>,
    blend_attachments: &'a [(Option<PipelineColorBlendState>, vk::ColorComponentFlags)],
    /// Set to [`None`] if `VK_DYNAMIC_STATE_BLEND_CONSTANTS` is enabled.
    blend_constants: Option<[NotNan<f32>; 4]>,
    /// Set to [`None`] if dynamic rendering is used.
    render_pass: Option<(vk::RenderPass, u32)>,
}

impl<'a> PipelineStaticState<'a> {
    fn generate_vertex_input_state<'b>(&self, alloc: &'b Bump, dynamic_state: &mut Vec<vk::DynamicState>) -> &'b vk::PipelineVertexInputStateCreateInfoBuilder<'b> {
        if self.input_binding_strides.is_none() {
            dynamic_state.push(vk::DynamicState::VERTEX_INPUT_BINDING_STRIDE);
        }

        let bindings = self.input_attributes.iter().enumerate().map(|(index, (_, _, rate))| {
            let mut binding = vk::VertexInputBindingDescription::builder()
                .binding(index as u32)
                .input_rate(*rate);
            if let Some(input_stride) = &self.input_binding_strides {
                binding = binding.stride(input_stride[index]);
            }
            binding.build()
        });
        let attributes = self.input_attributes.iter().enumerate().map(|(index, (location, format, _))| {
            vk::VertexInputAttributeDescription::builder()
                .location(*location)
                .binding(index as u32)
                .format(*format)
                .offset(0)
                .build()
        });

        let state = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(alloc.alloc_slice_fill_iter(bindings))
            .vertex_attribute_descriptions(alloc.alloc_slice_fill_iter(attributes));

        alloc.alloc(state)
    }

    fn generate_input_assembly_state<'b>(&self, alloc: &'b Bump, dynamic_state: &mut Vec<vk::DynamicState>) -> &'b vk::PipelineInputAssemblyStateCreateInfoBuilder<'b> {
        let mut state = vk::PipelineInputAssemblyStateCreateInfo::builder();
        if let Some(topology) = &self.primitive_topology {
            state = state.topology(*topology);
        } else {
            dynamic_state.push(vk::DynamicState::PRIMITIVE_TOPOLOGY);
        }
        if let Some(primitive_restart_enable) = &self.primitive_restart_enable {
            state = state.primitive_restart_enable(*primitive_restart_enable);
        } else {
            dynamic_state.push(vk::DynamicState::PRIMITIVE_RESTART_ENABLE);
        }
        alloc.alloc(state)
    }

    fn generate_rasterization_state<'b>(&self, alloc: &'b Bump, dynamic_state: &mut Vec<vk::DynamicState>) -> &'b vk::PipelineRasterizationStateCreateInfoBuilder<'b> {
        let mut state = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(self.depth_clamp_enable)
            .rasterizer_discard_enable(self.rasterizer_discard_enable)
            .polygon_mode(self.polygon_mode);
        if let Some(cull_mode) = &self.cull_mode {
            state = state.cull_mode(*cull_mode);
        } else {
            dynamic_state.push(vk::DynamicState::CULL_MODE);
        }
        if let Some(front_face) = &self.front_face {
            state = state.front_face(*front_face);
        } else {
            dynamic_state.push(vk::DynamicState::FRONT_FACE);
        }
        if let Some(line_width) = &self.line_width {
            state = state.line_width(line_width.into_inner());
        } else {
            dynamic_state.push(vk::DynamicState::LINE_WIDTH);
        }
        alloc.alloc(state)
    }

    fn generate_depth_stencil_state<'b>(&self, alloc: &'b Bump, dynamic_state: &mut Vec<vk::DynamicState>) -> &'b vk::PipelineDepthStencilStateCreateInfoBuilder<'b> {
        let mut state = vk::PipelineDepthStencilStateCreateInfo::builder();
        if let Some(depth_test_enable) = &self.depth_test_enable {
            state = state.depth_test_enable(*depth_test_enable);
        } else {
            dynamic_state.push(vk::DynamicState::DEPTH_TEST_ENABLE);
        }
        if let Some(depth_write_enable) = &self.depth_write_enable {
            state = state.depth_write_enable(*depth_write_enable);
        } else {
            dynamic_state.push(vk::DynamicState::DEPTH_WRITE_ENABLE);
        }
        if let Some(depth_compare_op) = &self.depth_compare_op {
            state = state.depth_compare_op(*depth_compare_op);
        } else {
            dynamic_state.push(vk::DynamicState::DEPTH_COMPARE_OP);
        }
        if let Some(stencil_test_enable) = &self.stencil_test_enable {
            state = state.stencil_test_enable(*stencil_test_enable);
        } else {
            dynamic_state.push(vk::DynamicState::STENCIL_TEST_ENABLE);
        }
        let (mut front, mut back) = (vk::StencilOpState::builder(), vk::StencilOpState::builder());
        if let Some((stencil_op_front, stencil_op_back)) = &self.stencil_op {
            front = front.fail_op(stencil_op_front.fail_op)
                .pass_op(stencil_op_front.pass_op)
                .depth_fail_op(stencil_op_front.depth_fail_op)
                .compare_op(stencil_op_front.compare_op);
            back = back.fail_op(stencil_op_back.fail_op)
                .pass_op(stencil_op_back.pass_op)
                .depth_fail_op(stencil_op_back.depth_fail_op)
                .compare_op(stencil_op_back.compare_op);
        } else {
            dynamic_state.push(vk::DynamicState::STENCIL_OP);
        }
        if let Some((compare_mask_front, compare_mask_back)) = &self.stencil_compare_mask {
            front = front.compare_mask(*compare_mask_front);
            back = back.compare_mask(*compare_mask_back);
        } else {
            dynamic_state.push(vk::DynamicState::STENCIL_COMPARE_MASK);
        }
        if let Some((write_mask_front, write_mask_back)) = &self.stencil_write_mask {
            front = front.write_mask(*write_mask_front);
            back = back.write_mask(*write_mask_back);
        } else {
            dynamic_state.push(vk::DynamicState::STENCIL_WRITE_MASK);
        }
        if let Some((reference_front, reference_back)) = &self.stencil_reference {
            front = front.reference(*reference_front);
            back = back.reference(*reference_back);
        } else {
            dynamic_state.push(vk::DynamicState::STENCIL_REFERENCE);
        }
        state = state.front(front.build()).back(back.build());
        alloc.alloc(state)
    }

    fn generate_color_blend_state<'b>(&self, alloc: &'b Bump, dynamic_state: &mut Vec<vk::DynamicState>) -> &'b vk::PipelineColorBlendStateCreateInfoBuilder<'b> {
        let mut state = vk::PipelineColorBlendStateCreateInfo::builder();
        if let Some(logic_op) = &self.logic_op {
            state = state.logic_op_enable(true)
                .logic_op(*logic_op);
        } else {
            state = state.logic_op_enable(false);
        }
        let attachments = alloc.alloc_slice_fill_iter(self.blend_attachments.iter().map(|(blend_state, write_mask)| {
            let mut blend = vk::PipelineColorBlendAttachmentState::builder();
            if let Some(blend_state) = blend_state {
                blend = blend.blend_enable(true)
                    .src_color_blend_factor(blend_state.src_color_blend_factor)
                    .dst_color_blend_factor(blend_state.dst_color_blend_factor)
                    .color_blend_op(blend_state.color_blend_op)
                    .src_alpha_blend_factor(blend_state.src_alpha_blend_factor)
                    .dst_alpha_blend_factor(blend_state.dst_alpha_blend_factor)
                    .alpha_blend_op(blend_state.alpha_blend_op);
            } else {
                blend = blend.blend_enable(false);
            }
            blend.color_write_mask(*write_mask).build()
        }));
        state = state.attachments(attachments);
        if let Some(blend_constants) = &self.blend_constants {
            state = state.blend_constants(unsafe {
                // Safe because NotNan is repr(transparent)
                std::mem::transmute(*blend_constants)
            });
        } else {
            dynamic_state.push(vk::DynamicState::BLEND_CONSTANTS);
        }
        alloc.alloc(state)
    }
}

#[derive(Copy, Clone, PartialEq, Hash, Debug)]
struct PipelineDynamicStateStencilOp {
    fail_op: vk::StencilOp,
    pass_op: vk::StencilOp,
    depth_fail_op: vk::StencilOp,
    compare_op: vk::CompareOp,
}

pub(super) struct ShaderStageInfo {
    pub stage: vk::ShaderStageFlags,
    pub module: vk::ShaderModule,
    pub entry: CString,
}

id_type!(GraphicsPipeline, GraphicsPipeline::get_id);