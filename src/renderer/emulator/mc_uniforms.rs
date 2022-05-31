//! Structs used to process minecrafts uniforms and samplers

use ash::prelude::VkResult;
use ash::vk;

use crate::prelude::*;

#[repr(u32)]
pub enum McUniform {
    ModelViewMatrix(Mat4f32),
    ProjectionMatrix(Mat4f32),
    InverseViewRotationMatrix(Mat4f32),
    TextureMatrix(Mat4f32),
    ScreenSize(Vec2f32),
    ColorModulator(Vec4f32),
    Light0Direction(Vec3f32),
    Light1Direction(Vec3f32),
    Fog {
        color: Vec4f32,
        start: f32,
        end: f32,
        shape: u32,
    },
    LineWidth(f32),
    GameTime(f32),
    ChunkOffset(Vec3f32),
}
// Sanity check
const_assert_eq!(std::mem::size_of::<McUniform>(), 68);

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct Set0Binding0 {
    #[allow(unused)]
    projection_matrix: Mat4f32,

    #[allow(unused)]
    fog_color: Vec4f32,

    #[allow(unused)]
    fog_start: f32,

    #[allow(unused)]
    fog_end: f32,

    #[allow(unused)]
    game_time: f32,

    _padding0: [u8; 4],

    #[allow(unused)]
    fog_shape: u32,

    _padding1: [u8; 12],

    #[allow(unused)]
    screen_size: Vec2f32,

    _padding2: [u8; 8],
}
// Sanity checks
const_assert_eq!(std::mem::size_of::<Set0Binding0>(), 128);
const_assert_eq!(std::mem::size_of::<Set0Binding0>() % 16, 0); // std140 size must be multiple of vec4

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct Set1Binding0 {
    #[allow(unused)]
    inverse_view_rotation_matrix: Mat4f32,

    #[allow(unused)]
    light_0_direction: Vec3f32,

    _padding0: [u8; 4],

    #[allow(unused)]
    light_1_direction: Vec3f32,

    _padding1: [u8; 4],

    #[allow(unused)]
    color_modulator: Vec4f32,
}
// Sanity checks
const_assert_eq!(std::mem::size_of::<Set1Binding0>(), 112);
const_assert_eq!(std::mem::size_of::<Set1Binding0>() % 16, 0); // std140 size must be multiple of vec4

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct PushConstants {
    #[allow(unused)]
    model_view_matrix: Mat4f32,

    #[allow(unused)]
    chunk_offset: Vec3f32,

    _padding0: [u8; 4],
}
// Sanity checks
const_assert_eq!(std::mem::size_of::<PushConstants>(), 80);
const_assert_eq!(std::mem::size_of::<PushConstants>() % 16, 0); // For consistency reasons

pub enum DescriptorBindingInfo {
    /// A sampled image descriptor.
    Image {
        /// The array size
        count: u32,
    },

    /// A uniform buffer descriptor.
    UniformBuffer {
        /// The minimum size needed for buffer memory.
        buffer_size: usize,

        /// If true a inline uniform block should be preferred if supported.
        should_inline: bool,
    }
}

impl DescriptorBindingInfo {
    pub fn as_descriptor_set_layout_binding(&self, binding: u32, inline_block_supported: bool) -> vk::DescriptorSetLayoutBinding {
        let result = vk::DescriptorSetLayoutBinding::builder()
            .binding(binding)
            .stage_flags(vk::ShaderStageFlags::ALL);

        match self {
            DescriptorBindingInfo::Image { count } => {
                result
                    .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                    .descriptor_count(*count)
                    .build()
            },
            DescriptorBindingInfo::UniformBuffer { buffer_size, should_inline } => {
                if *should_inline && inline_block_supported {
                    result
                        .descriptor_type(vk::DescriptorType::INLINE_UNIFORM_BLOCK)
                        .descriptor_count(*buffer_size as u32)
                        .build()
                } else {
                    result
                        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                        .descriptor_count(1)
                        .build()
                }
            }
        }
    }

    pub const fn get_type(&self, inline_block_supported: bool) -> vk::DescriptorType {
        match self {
            DescriptorBindingInfo::Image { .. } => {
                vk::DescriptorType::SAMPLED_IMAGE
            }
            DescriptorBindingInfo::UniformBuffer { should_inline, .. } => {
                if *should_inline && inline_block_supported {
                    vk::DescriptorType::INLINE_UNIFORM_BLOCK
                } else {
                    vk::DescriptorType::UNIFORM_BUFFER
                }
            }
        }
    }
}

pub const SET0_DESCRIPTOR_INFOS: [DescriptorBindingInfo; 1] = [
    DescriptorBindingInfo::UniformBuffer {
        buffer_size: std::mem::size_of::<Set0Binding0>(),
        should_inline: true
    },
];

pub const SET1_DESCRIPTOR_INFOS: [DescriptorBindingInfo; 4] = [
    DescriptorBindingInfo::UniformBuffer {
        buffer_size: std::mem::size_of::<Set1Binding0>(),
        should_inline: false
    },
    DescriptorBindingInfo::Image {
        count: 1
    },
    DescriptorBindingInfo::Image {
        count: 1
    },
    DescriptorBindingInfo::Image {
        count: 1
    },
];

pub fn generate_set0_layout(device: &DeviceContext) -> VkResult<vk::DescriptorSetLayout> {
    generate_set_layout(device, SET0_DESCRIPTOR_INFOS.as_ref())
}

pub fn generate_set1_layout(device: &DeviceContext) -> VkResult<vk::DescriptorSetLayout> {
    generate_set_layout(device, SET1_DESCRIPTOR_INFOS.as_ref())
}

fn generate_set_layout(device: &DeviceContext, binding_info: &[DescriptorBindingInfo]) -> VkResult<vk::DescriptorSetLayout> {
    let bindings: Box<_> = binding_info.iter().enumerate().map(|(binding, info)| {
        info.as_descriptor_set_layout_binding(binding as u32, false)
    }).collect();

    let info = vk::DescriptorSetLayoutCreateInfo::builder()
        .bindings(bindings.as_ref());

    unsafe {
        device.vk().create_descriptor_set_layout(&info, None)
    }
}

/// Tracks the state of minecraft uniforms for rendering
pub(super) struct McUniformState {
    set_0_cache: vk::DescriptorSet,
    set_1_cache: vk::DescriptorSet,
    push_constants: PushConstants,
    set_0_binding_0: Set0Binding0,
    set_1_binding_0: Set1Binding0,
}

impl McUniformState {
    pub fn write(&mut self, uniform: &McUniform) {
        todo!()
    }
}