//! Structs used to process minecrafts uniforms and samplers

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex, Weak};
use ash::vk;

use crate::prelude::*;
use crate::{define_uuid_type, to_bytes_body};

define_uuid_type!(pub, ShaderId);

pub trait ShaderDropListener {
    fn on_shader_drop(&self, id: ShaderId);
}

pub struct Shader {
    id: ShaderId,
    vertex_format: VertexFormat,
    weak: Weak<Self>,
    listeners: Mutex<HashMap<UUID, Weak<dyn ShaderDropListener + Send + Sync>>>,
}

impl Shader {
    pub fn new(vertex_format: VertexFormat) -> Arc<Self> {
        Arc::new_cyclic(|weak| {
            Self {
                id: ShaderId::new(),
                vertex_format,
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

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct DevUniform {
    #[allow(unused)]
    projection_matrix: Mat4f32,

    #[allow(unused)]
    model_view_matrix: Mat4f32,

    #[allow(unused)]
    chunk_offset: Vec3f32,

    _padding0: [u8; 4],
}
const_assert_eq!(std::mem::size_of::<DevUniform>(), 144);
const_assert_eq!(std::mem::size_of::<DevUniform>() % 16, 0); // std140 size must be multiple of vec4

unsafe impl ToBytes for DevUniform { to_bytes_body!(); }

#[derive(Copy, Clone)]
pub struct VertexFormatEntry {
    pub offset: u32,
    pub format: vk::Format,
}

#[derive(Copy, Clone)]
pub struct VertexFormat {
    pub stride: u32,
    pub position: VertexFormatEntry,
}