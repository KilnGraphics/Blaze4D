//! Structs used to process minecrafts uniforms and samplers

use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};
use std::sync::{Arc, Mutex, Weak};
use ash::vk;
use crate::define_uuid_type;

use crate::prelude::*;

define_uuid_type!(pub, ShaderId);

pub trait ShaderDropListener {
    fn on_shader_drop(&self, id: ShaderId);
}

pub struct Shader {
    id: ShaderId,
    vertex_format: VertexFormat,
    used_uniforms: McUniform,
    weak: Weak<Self>,
    listeners: Mutex<HashMap<UUID, Weak<dyn ShaderDropListener + Send + Sync>>>,
}

impl Shader {
    pub fn new(vertex_format: VertexFormat, used_uniforms: McUniform) -> Arc<Self> {
        Arc::new_cyclic(|weak| {
            Self {
                id: ShaderId::new(),
                vertex_format,
                used_uniforms,
                weak: weak.clone(),
                listeners: Mutex::new(HashMap::new()),
            }
        })
    }

    pub fn get_id(&self) -> ShaderId {
        self.id
    }

    pub fn get_vertex_format(&self) -> &VertexFormat {
        &self.vertex_format
    }

    pub fn get_used_uniforms(&self) -> McUniform {
        self.used_uniforms
    }

    /// Registers a drop listener to this shader. If this shader is dropped the listener will be called.
    ///
    /// The returned [`ShaderListener`] is used keep track of the liveliness of the listener. If it is
    /// dropped the listener will be removed from the shader.
    pub fn register_drop_listener(&self, listener: &Arc<dyn ShaderDropListener + Send + Sync>) -> ShaderListener {
        let id = UUID::new();

        let mut guard = self.listeners.lock().unwrap();
        guard.insert(id, Arc::downgrade(listener));

        ShaderListener {
            shader: self.weak.clone(),
            listener_id: id,
        }
    }

    /// Called by [`ShaderRef`] when it is dropped to remove any dangling listeners.
    fn remove_listener(&self, id: UUID) {
        let mut guard = self.listeners.lock().unwrap();
        guard.remove(&id);
    }
}

pub struct ShaderListener {
    shader: Weak<Shader>,
    listener_id: UUID,
}

impl Drop for ShaderListener {
    fn drop(&mut self) {
        if let Some(shader) = self.shader.upgrade() {
            shader.remove_listener(self.listener_id)
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct McUniform(u64);

impl McUniform {
    #[inline]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    #[inline]
    pub const fn as_raw(&self) -> u64 {
        self.0
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.0 == Self::empty().0
    }

    #[inline]
    pub const fn intersects(&self, other: &Self) -> bool {
        !Self(self.0 & other.0).is_empty()
    }

    #[inline]
    pub const fn contains(&self, other: &Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub const MODEL_VIEW_MATRIX: Self = Self::from_raw(1u64);
    pub const PROJECTION_MATRIX: Self = Self::from_raw(1u64 << 1);
    pub const INVERSE_VIEW_ROTATION_MATRIX: Self = Self::from_raw(1u64 << 2);
    pub const TEXTURE_MATRIX: Self = Self::from_raw(1u64 << 3);
    pub const SCREEN_SIZE: Self = Self::from_raw(1u64 << 4);
    pub const COLOR_MODULATOR: Self = Self::from_raw(1u64 << 5);
    pub const LIGHT0_DIRECTION: Self = Self::from_raw(1u64 << 6);
    pub const LIGHT1_DIRECTION: Self = Self::from_raw(1u64 << 7);
    pub const FOG_START: Self = Self::from_raw(1u64 << 8);
    pub const FOG_END: Self = Self::from_raw(1u64 << 9);
    pub const FOG_COLOR: Self = Self::from_raw(1u64 << 10);
    pub const FOG_SHAPE: Self = Self::from_raw(1u64 << 11);
    pub const LINE_WIDTH: Self = Self::from_raw(1u64 << 12);
    pub const GAME_TIME: Self = Self::from_raw(1u64 << 13);
    pub const CHUNK_OFFSET: Self = Self::from_raw(1u64 << 14);
}

impl BitOr for McUniform {
    type Output = McUniform;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for McUniform {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs
    }
}

impl BitAnd for McUniform {
    type Output = McUniform;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for McUniform {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs
    }
}

impl BitXor for McUniform {
    type Output = McUniform;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for McUniform {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs
    }
}

impl Not for McUniform {
    type Output = McUniform;

    #[inline]
    fn not(self) -> Self::Output {
        Self(self.0.not())
    }
}

#[derive(Copy, Clone, Debug)]
pub enum McUniformData {
    ModelViewMatrix(Mat4f32),
    ProjectionMatrix(Mat4f32),
    InverseViewRotationMatrix(Mat4f32),
    TextureMatrix(Mat4f32),
    ScreenSize(Vec2f32),
    ColorModulator(Vec4f32),
    Light0Direction(Vec3f32),
    Light1Direction(Vec3f32),
    FogStart(f32),
    FogEnd(f32),
    FogColor(Vec4f32),
    FogShape(u32),
    LineWidth(f32),
    GameTime(f32),
    ChunkOffset(Vec3f32),
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct DevUniform {
    #[allow(unused)]
    pub projection_matrix: Mat4f32,

    #[allow(unused)]
    pub model_view_matrix: Mat4f32,

    #[allow(unused)]
    pub chunk_offset: Vec3f32,

    _padding0: [u8; 4],
}
const_assert_eq!(std::mem::size_of::<DevUniform>(), 144);
const_assert_eq!(std::mem::size_of::<DevUniform>() % 16, 0); // std140 size must be multiple of vec4

#[derive(Copy, Clone, Debug)]
pub struct VertexFormatEntry {
    pub offset: u32,
    pub format: vk::Format,
}

#[derive(Copy, Clone, Debug)]
pub struct VertexFormat {
    pub stride: u32,
    pub position: VertexFormatEntry,
    pub normal: Option<VertexFormatEntry>,
    pub color: Option<VertexFormatEntry>,
    pub uv0: Option<VertexFormatEntry>,
    pub uv1: Option<VertexFormatEntry>,
    pub uv2: Option<VertexFormatEntry>,
}