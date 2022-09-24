use std::cell::UnsafeCell;
use std::cmp::Ordering;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::ffi::CString;
use std::hash::{BuildHasher, Hash, Hasher};
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use ash::vk;
use ash::vk::CommandBuffer;
use bumpalo::Bump;
use ordered_float::NotNan;
use crate::allocator::{Allocation, HostAccess};

use super::share::Share2;
use crate::define_uuid_type;

use crate::prelude::*;
use crate::renderer::emulator::BBox;

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

pub struct PipelineLayout {
    share: Arc<Share2>,
    handle: vk::PipelineLayout,
    descriptor_set_layouts: Box<[vk::DescriptorSetLayout]>,
    descriptor_sets: Box<[Box<[DescriptorBinding]>]>,
}

impl PipelineLayout {
    pub(super) fn new(share: Arc<Share2>, descriptor_sets: Box<[Box<[DescriptorBinding]>]>) -> Self {
        let device = share.get_device();
        let mut alloc = Bump::new();

        let mut descriptor_set_layouts = Vec::with_capacity(descriptor_sets.len());
        for bindings in descriptor_sets.iter() {
            alloc.reset();
            let set_bindings = alloc.alloc_slice_fill_iter(bindings.iter().map(DescriptorBinding::get_vk_descriptor_set_layout_binding));

            let info = vk::DescriptorSetLayoutCreateInfo::builder()
                .flags(vk::DescriptorSetLayoutCreateFlags::PUSH_DESCRIPTOR_KHR)
                .bindings(set_bindings);

            let set_layout =unsafe {
                device.vk().create_descriptor_set_layout(&info, None)
            }.map_err(|err| {
                for layout in &descriptor_set_layouts {
                    unsafe { device.vk().destroy_descriptor_set_layout(*layout, None) };
                }
                err
            }).expect("Failed to create descriptor set layout in PipelineLayout::new");

            descriptor_set_layouts.push(set_layout);
        }

        let info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&descriptor_set_layouts);

        let handle = unsafe {
            device.vk().create_pipeline_layout(&info, None)
        }.map_err(|err| {
            for layout in &descriptor_set_layouts {
                unsafe { device.vk().destroy_descriptor_set_layout(*layout, None) };
            }
            err
        }).expect("Failed to create pipeline layout in PipelineLayout::new");

        Self {
            share,
            handle,
            descriptor_set_layouts: descriptor_set_layouts.into_boxed_slice(),
            descriptor_sets,
        }
    }

    pub(super) fn get_handle(&self) -> vk::PipelineLayout {
        self.handle
    }

    pub(super) fn get_descriptor_sets(&self) -> &[Box<[DescriptorBinding]>] {
        &self.descriptor_sets
    }
}

impl Drop for PipelineLayout {
    fn drop(&mut self) {
        let device = self.share.get_device();
        unsafe {
            device.vk().destroy_pipeline_layout(self.handle, None);
            for layout in self.descriptor_set_layouts.iter() {
                device.vk().destroy_descriptor_set_layout(*layout, None);
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Hash, Debug)]
pub struct DescriptorBinding {
    binding: u32,
    descriptor_type: DescriptorType,
    stage_flags: vk::ShaderStageFlags,
}

impl DescriptorBinding {
    pub fn get_vk_descriptor_set_layout_binding(&self) -> vk::DescriptorSetLayoutBinding {
        vk::DescriptorSetLayoutBinding::builder()
            .binding(self.binding)
            .descriptor_type(self.descriptor_type.get_vk_descriptor_type())
            .descriptor_count(self.descriptor_type.get_vk_descriptor_count())
            .stage_flags(self.stage_flags)
            .build()
    }
}

#[derive(Copy, Clone, PartialEq, Hash, Debug)]
pub enum DescriptorType {
    CombinedImageSampler {
        count: u32,
    },
    InlineUniformBlock {
        size: u32,
    }
}

impl DescriptorType {
    pub fn get_vk_descriptor_type(&self) -> vk::DescriptorType {
        match self {
            DescriptorType::CombinedImageSampler { .. } => vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            DescriptorType::InlineUniformBlock { .. } => vk::DescriptorType::INLINE_UNIFORM_BLOCK,
        }
    }

    pub fn get_vk_descriptor_count(&self) -> u32 {
        match self {
            DescriptorType::CombinedImageSampler { count, .. } |
            DescriptorType::InlineUniformBlock { size: count, .. } => *count,
        }
    }
}

define_uuid_type!(pub, GraphicsPipelineId);

pub struct GraphicsPipeline {
    share: Arc<Share2>,
    id: GraphicsPipelineId,
    handle: vk::Pipeline,
    #[allow(unused)] // Only needed to keep the object alive
    layout: Arc<PipelineLayout>,
}

impl GraphicsPipeline {
    pub(super) fn new<S: PipelineStaticState2>(share: Arc<Share2>, state: &S, shader_stages: &[ShaderStageInfo], layout: Arc<PipelineLayout>) -> Self {
        let bump = Bump::new();

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
            .vertex_input_state(state.generate_vertex_input_state(&bump))
            .input_assembly_state(state.generate_input_assembly_state(&bump))
            .viewport_state(&viewport_state)
            .rasterization_state(state.generate_rasterization_state(&bump))
            .multisample_state(&multisample_state)
            .depth_stencil_state(state.generate_depth_stencil_state(&bump))
            .color_blend_state(state.generate_color_blend_state(&bump));

        let mut dynamic_state = Vec::with_capacity(32);
        dynamic_state.push(vk::DynamicState::VIEWPORT);
        dynamic_state.push(vk::DynamicState::SCISSOR);
        state.collect_dynamic_state(&mut dynamic_state);

        let dynamic_state = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&dynamic_state);

        info = info.dynamic_state(&dynamic_state)
            .layout(layout.get_handle());

        let handle = *unsafe {
            share.get_device().vk().create_graphics_pipelines(vk::PipelineCache::null(), std::slice::from_ref(&info), None)
        }.expect("Failed to create emulator graphics pipeline").first().unwrap();

        Self {
            share,
            id: GraphicsPipelineId::new(),
            handle,
            layout
        }
    }

    pub fn get_id(&self) -> GraphicsPipelineId {
        self.id
    }
}

impl Drop for GraphicsPipeline {
    fn drop(&mut self) {
        unsafe {
            self.share.get_device().vk().destroy_pipeline(self.handle, None);
        }
    }
}

pub trait PipelineStaticState2 {
    fn generate_vertex_input_state<'a>(&self, alloc: &'a Bump) -> &'a vk::PipelineVertexInputStateCreateInfo;

    fn generate_input_assembly_state<'a>(&self, alloc: &'a Bump) -> &'a vk::PipelineInputAssemblyStateCreateInfo;

    fn generate_rasterization_state<'a>(&self, alloc: &'a Bump) -> &'a vk::PipelineRasterizationStateCreateInfo;

    fn generate_depth_stencil_state<'a>(&self, alloc: &'a Bump) -> &'a vk::PipelineDepthStencilStateCreateInfo;

    fn generate_color_blend_state<'a>(&self, alloc: &'a Bump) -> &'a vk::PipelineColorBlendStateCreateInfo;

    /// Collects all used dynamic state in the provided [`Vec`].
    ///
    /// This must only include dynamic state configurable by this trait. In particular this means
    /// `VK_DYNAMIC_STATE_VIEWPORT` and `VK_DYNAMIC_STATE_SCISSOR` must not be added by this
    /// function.
    fn collect_dynamic_state(&self, dynamic_state: &mut Vec<vk::DynamicState>);
}

pub trait PipelineDynamicState2 {
    fn setup_pipeline(&self, device: &DeviceContext, cmd: vk::CommandBuffer, tracker: &mut DynamicStateTracker);
}

pub trait PipelineStateProvider {
    fn get_input_bindings(&self) -> &[PipelineVertexInputBinding];

    fn get_primitive_topology(&self) -> vk::PrimitiveTopology;

    fn get_primitive_restart_enable(&self) -> bool;

    fn get_viewport(&self) -> (Vec2f32, Vec2f32);

    fn get_scissor(&self) -> (Vec2u32, Vec2u32);

    fn get_depth_clamp_enable(&self) -> bool;

    fn get_rasterizer_discard_enable(&self) -> bool;

    fn get_polygon_mode(&self) -> vk::PolygonMode;

    fn get_cull_mode(&self) -> vk::CullModeFlags;

    fn get_front_face(&self) -> vk::FrontFace;

    fn get_line_width(&self) -> f32;

    fn get_depth_test(&self) -> Option<PipelineDepthTest>;

    fn get_stencil_test(&self) -> Option<(PipelineStencilTest, PipelineStencilTest)>;

    fn get_color_blending(&self) -> &[PipelineColorBlending];

    fn get_blend_constants(&self) -> [f32; 4];
}

#[derive(Copy, Clone, PartialEq, Default, Hash, Debug)]
pub struct PipelineVertexInputBinding {
    pub location: u32,
    pub stride: u32,
    pub format: vk::Format,
    pub input_rate: vk::VertexInputRate,
}

#[derive(Copy, Clone, PartialEq, Default, Hash, Debug)]
pub struct PipelineDepthTest {
    pub write_enable: bool,
    pub compare_op: vk::CompareOp,
}

#[derive(Copy, Clone, PartialEq, Default, Hash, Debug)]
pub struct PipelineStencilTest {
    pub fail_op: vk::StencilOp,
    pub pass_op: vk::StencilOp,
    pub depth_fail_op: vk::StencilOp,
    pub compare_op: vk::CompareOp,
    pub compare_mask: u32,
    pub write_mask: u32,
    pub reference: u32,
}

#[derive(Copy, Clone, PartialEq, Default, Hash, Debug)]
pub struct PipelineColorBlending {
    pub src_color_blend_factor: vk::BlendFactor,
    pub dst_color_blend_factor: vk::BlendFactor,
    pub color_blend_op: vk::BlendOp,
    pub src_alpha_blend_factor: vk::BlendFactor,
    pub dst_alpha_blend_factor: vk::BlendFactor,
    pub alpha_blend_op: vk::BlendOp,
}

pub struct DynamicStateTracker {
    pub viewport: Option<(Vec2f32, Vec2f32)>,
    pub scissor: Option<(Vec2u32, Vec2u32)>,
    pub primitive_topology: Option<vk::PrimitiveTopology>,
    pub primitive_restart_enable: Option<bool>,
    pub cull_mode: Option<vk::CullModeFlags>,
    pub front_face: Option<vk::FrontFace>,
    pub line_width: Option<NotNan<f32>>,
    pub depth_test_enable: Option<bool>,
    pub depth_write_enable: Option<bool>,
    pub depth_compare_op: Option<vk::CompareOp>,
    pub stencil_test_enable: Option<bool>,
    pub stencil_op: Option<(DynamicStateStencilOp, DynamicStateStencilOp)>,
    pub stencil_compare_mask: Option<(u32, u32)>,
    pub stencil_write_mask: Option<(u32, u32)>,
    pub stencil_reference: Option<(u32, u32)>,
    pub blend_constants: Option<[NotNan<f32>; 4]>,
}

impl DynamicStateTracker {
    pub(super) fn new() -> Self {
        Self {
            viewport: None,
            scissor: None,
            primitive_topology: None,
            primitive_restart_enable: None,
            cull_mode: None,
            front_face: None,
            line_width: None,
            depth_test_enable: None,
            depth_write_enable: None,
            depth_compare_op: None,
            stencil_test_enable: None,
            stencil_op: None,
            stencil_compare_mask: None,
            stencil_write_mask: None,
            stencil_reference: None,
            blend_constants: None
        }
    }
}

#[derive(Copy, Clone, PartialEq, Default, Hash, Debug)]
pub struct DynamicStateStencilOp {
    pub fail_op: vk::StencilOp,
    pub pass_op: vk::StencilOp,
    pub depth_fail_op: vk::StencilOp,
    pub compare_op: vk::CompareOp,
}

impl DynamicStateStencilOp {
    fn from_stencil_test(src: &PipelineStencilTest) -> Self {
        Self {
            fail_op: src.fail_op,
            pass_op: src.pass_op,
            depth_fail_op: src.depth_fail_op,
            compare_op: src.compare_op,
        }
    }
}

pub trait PipelineDynamicStateConfiguration {
    const INPUT_BINDING_STRIDE: bool;
    const PRIMITIVE_TOPOLOGY: bool;
    const PRIMITIVE_RESTART_ENABLE: bool;
    const DEPTH_CLAMP_ENABLE: bool;
    const RASTERIZER_DISCARD_ENABLE: bool;
    const POLYGON_MODE: bool;
    const CULL_MODE: bool;
    const FRONT_FACE: bool;
    const LINE_WIDTH: bool;
    const DEPTH_TEST_ENABLE: bool;
    const DEPTH_WRITE_ENABLE: bool;
    const DEPTH_COMPARE_OP: bool;
    const STENCIL_TEST_ENABLE: bool;
    const STENCIL_OP: bool;
    const STENCIL_COMPARE_MASK: bool;
    const STENCIL_WRITE_MASK: bool;
    const STENCIL_REFERENCE: bool;
    const BLEND_CONSTANTS: bool;
    const USED_DYNAMIC_STATE: &'static [vk::DynamicState];
}

pub struct ConstPipelineDynamicState<'a, C: PipelineDynamicStateConfiguration> {
    viewport: (Vec2f32, Vec2f32),
    scissor: (Vec2u32, Vec2u32),
    input_binding_stride: Option<&'a [u32]>,
    primitive_topology: vk::PrimitiveTopology,
    primitive_restart_enable: bool,
    depth_clamp_enable: bool,
    rasterizer_discard_enable: bool,
    polygon_mode: vk::PolygonMode,
    cull_mode: vk::CullModeFlags,
    front_face: vk::FrontFace,
    line_width: f32,
    depth_test: Option<PipelineDepthTest>,
    stencil_test: Option<(PipelineStencilTest, PipelineStencilTest)>,
    blend_constants: [f32; 4],
    _phantom: PhantomData<C>,
}

impl<'a, C: PipelineDynamicStateConfiguration> ConstPipelineDynamicState<'a, C> {
    pub fn from_state_provider<P: PipelineStateProvider>(alloc: &'a Bump, provider: &P) -> Self {
        let mut state = Self {
            viewport: provider.get_viewport(),
            scissor: provider.get_scissor(),

            input_binding_stride: None,
            primitive_topology: Default::default(),
            primitive_restart_enable: false,
            depth_clamp_enable: false,
            rasterizer_discard_enable: false,
            polygon_mode: Default::default(),
            cull_mode: Default::default(),
            front_face: Default::default(),
            line_width: 0.0,
            depth_test: None,
            stencil_test: None,
            blend_constants: Default::default(),
            _phantom: PhantomData,
        };

        if C::INPUT_BINDING_STRIDE {
            let iter = provider.get_input_bindings().iter().map(|b| b.stride);
            state.input_binding_stride = Some(alloc.alloc_slice_fill_iter(iter));
        }
        if C::PRIMITIVE_TOPOLOGY {
            state.primitive_topology = provider.get_primitive_topology();
        }
        if C::PRIMITIVE_RESTART_ENABLE {
            state.primitive_restart_enable = provider.get_primitive_restart_enable();
        }
        if C::DEPTH_CLAMP_ENABLE {
            state.depth_clamp_enable = provider.get_depth_clamp_enable();
        }
        if C::RASTERIZER_DISCARD_ENABLE {
            state.rasterizer_discard_enable = provider.get_rasterizer_discard_enable();
        }
        if C::POLYGON_MODE {
            state.polygon_mode = provider.get_polygon_mode();
        }
        if C::CULL_MODE {
            state.cull_mode = provider.get_cull_mode();
        }
        if C::FRONT_FACE {
            state.front_face = provider.get_front_face();
        }
        if C::LINE_WIDTH {
            state.line_width = provider.get_line_width();
        }
        if C::DEPTH_TEST_ENABLE || C::DEPTH_WRITE_ENABLE || C::DEPTH_COMPARE_OP {
            state.depth_test = provider.get_depth_test();
        }
        if C::STENCIL_TEST_ENABLE || C::STENCIL_OP || C::STENCIL_COMPARE_MASK || C::STENCIL_WRITE_MASK || C::STENCIL_REFERENCE {
            state.stencil_test = provider.get_stencil_test();
        }
        if C::BLEND_CONSTANTS {
            state.blend_constants = provider.get_blend_constants();
        }

        state
    }
}

impl<'a, C: PipelineDynamicStateConfiguration> PipelineDynamicState2 for ConstPipelineDynamicState<'a, C> {
    fn setup_pipeline(&self, device: &DeviceContext, cmd: CommandBuffer, tracker: &mut DynamicStateTracker) {
        if tracker.viewport != Some(self.viewport) {
            let viewport = vk::Viewport {
                x: self.viewport.0.x,
                y: self.viewport.0.y,
                width: self.viewport.1.x,
                height: self.viewport.1.y,
                min_depth: 0.0,
                max_depth: 1.0
            };
            unsafe { device.vk().cmd_set_viewport(cmd, 0, std::slice::from_ref(&viewport)) };
            tracker.viewport = Some(self.viewport);
        }

        if tracker.scissor != Some(self.scissor) {
            let scissor = vk::Rect2D {
                offset: vk::Offset2D { x: self.scissor.0.x as i32, y: self.scissor.0.y as i32 },
                extent: vk::Extent2D { width: self.scissor.1.x, height: self.scissor.1.y }
            };
            unsafe { device.vk().cmd_set_scissor(cmd, 0, std::slice::from_ref(&scissor)) };
            tracker.scissor = Some(self.scissor);
        }

        if C::LINE_WIDTH {
            let line_width = NotNan::new(self.line_width).unwrap();
            if tracker.line_width != Some(line_width) {
                unsafe { device.vk().cmd_set_line_width(cmd, self.line_width) };
                tracker.line_width = Some(line_width);
            }
        } else {
            tracker.line_width = None;
        }

        if C::BLEND_CONSTANTS {
            let blend_constants = [
                NotNan::new(self.blend_constants[0]).unwrap(),
                NotNan::new(self.blend_constants[1]).unwrap(),
                NotNan::new(self.blend_constants[2]).unwrap(),
                NotNan::new(self.blend_constants[3]).unwrap(),
            ];
            if tracker.blend_constants != Some(blend_constants) {
                unsafe { device.vk().cmd_set_blend_constants(cmd, &self.blend_constants) };
                tracker.blend_constants = Some(blend_constants);
            }
        } else {
            tracker.blend_constants = None;
        }

        if C::STENCIL_COMPARE_MASK {
            let compare_mask = self.stencil_test.map(|(f, b)| (f.compare_mask, b.compare_mask)).unwrap_or_default();
            if tracker.stencil_compare_mask != Some(compare_mask) {
                unsafe {
                    device.vk().cmd_set_stencil_compare_mask(cmd, vk::StencilFaceFlags::FRONT, compare_mask.0);
                    device.vk().cmd_set_stencil_compare_mask(cmd, vk::StencilFaceFlags::BACK, compare_mask.1);
                }
                tracker.stencil_compare_mask = Some(compare_mask)
            }
        } else {
            tracker.stencil_compare_mask = None;
        }

        if C::STENCIL_WRITE_MASK {
            let write_mask = self.stencil_test.map(|(f, b)| (f.write_mask, b.write_mask)).unwrap_or_default();
            if tracker.stencil_write_mask != Some(write_mask) {
                unsafe {
                    device.vk().cmd_set_stencil_write_mask(cmd, vk::StencilFaceFlags::FRONT, write_mask.0);
                    device.vk().cmd_set_stencil_write_mask(cmd, vk::StencilFaceFlags::BACK, write_mask.1);
                }
                tracker.stencil_write_mask = Some(write_mask)
            }
        } else {
            tracker.stencil_write_mask = None;
        }

        if C::STENCIL_REFERENCE {
            let reference = self.stencil_test.map(|(f, b)| (f.reference, b.reference)).unwrap_or_default();
            if tracker.stencil_reference != Some(reference) {
                unsafe {
                    device.vk().cmd_set_stencil_reference(cmd, vk::StencilFaceFlags::FRONT, reference.0);
                    device.vk().cmd_set_stencil_reference(cmd, vk::StencilFaceFlags::BACK, reference.1);
                }
                tracker.stencil_reference = Some(reference)
            }
        } else {
            tracker.stencil_reference = None;
        }

        if let Some(vk) = device.extended_dynamic_state_ext() {
            if C::PRIMITIVE_TOPOLOGY {
                if tracker.primitive_topology != Some(self.primitive_topology) {
                    unsafe { vk.cmd_set_primitive_topology(cmd, self.primitive_topology) };
                    tracker.primitive_topology = Some(self.primitive_topology);
                }
            } else {
                tracker.primitive_topology = None;
            }

            if C::CULL_MODE {
                if tracker.cull_mode != Some(self.cull_mode) {
                    unsafe { vk.cmd_set_cull_mode(cmd, self.cull_mode) };
                    tracker.cull_mode = Some(self.cull_mode);
                }
            } else {
                tracker.cull_mode = None;
            }

            if C::FRONT_FACE {
                if tracker.front_face != Some(self.front_face) {
                    unsafe { vk.cmd_set_front_face(cmd, self.front_face) };
                    tracker.front_face = Some(self.front_face);
                }
            } else {
                tracker.front_face = None;
            }

            if C::DEPTH_TEST_ENABLE {
                if tracker.depth_test_enable != Some(self.depth_test.is_some()) {
                    unsafe { vk.cmd_set_depth_test_enable(cmd, self.depth_test.is_some()) };
                    tracker.depth_test_enable = Some(self.depth_test.is_some());
                }
            } else {
                tracker.depth_test_enable = None;
            }

            if C::DEPTH_COMPARE_OP {
                let compare_op = self.depth_test.as_ref().map(|d| d.compare_op).unwrap_or(vk::CompareOp::ALWAYS);
                if tracker.depth_compare_op != Some(compare_op) {
                    unsafe { vk.cmd_set_depth_compare_op(cmd, compare_op) };
                    tracker.depth_compare_op = Some(compare_op);
                }
            } else {
                tracker.depth_compare_op = None;
            }

            if C::DEPTH_WRITE_ENABLE {
                let depth_write_enable = self.depth_test.as_ref().map(|d| d.write_enable).unwrap_or(false);
                if tracker.depth_write_enable != Some(depth_write_enable) {
                    unsafe { vk.cmd_set_depth_write_enable(cmd, depth_write_enable) };
                    tracker.depth_write_enable = Some(depth_write_enable);
                }
            } else {
                tracker.depth_write_enable = None;
            }

            if C::STENCIL_TEST_ENABLE {
                if tracker.stencil_test_enable != Some(self.stencil_test.is_some()) {
                    unsafe { vk.cmd_set_stencil_test_enable(cmd, self.stencil_test.is_some()) };
                    tracker.stencil_test_enable = Some(self.stencil_test.is_some());
                }
            } else {
                tracker.stencil_test_enable = None;
            }

            if C::STENCIL_OP {
                let stencil_op = self.stencil_test.as_ref().map(|(f, b)|
                    (DynamicStateStencilOp::from_stencil_test(f), DynamicStateStencilOp::from_stencil_test(b))
                ).unwrap_or_default();
                if tracker.stencil_op != Some(stencil_op) {
                    unsafe {
                        vk.cmd_set_stencil_op(cmd, vk::StencilFaceFlags::FRONT, stencil_op.0.fail_op, stencil_op.0.pass_op, stencil_op.0.depth_fail_op, stencil_op.0.compare_op);
                        vk.cmd_set_stencil_op(cmd, vk::StencilFaceFlags::BACK, stencil_op.1.fail_op, stencil_op.1.pass_op, stencil_op.1.depth_fail_op, stencil_op.1.compare_op);
                    }
                    tracker.stencil_op = Some(stencil_op);
                }
            } else {
                tracker.stencil_op = None;
            }
        }
        todo!()
    }
}

pub struct ShaderStageInfo {
    pub stage: vk::ShaderStageFlags,
    pub module: vk::ShaderModule,
    pub entry: CString,
}

id_type!(GraphicsPipeline, GraphicsPipeline::get_id);