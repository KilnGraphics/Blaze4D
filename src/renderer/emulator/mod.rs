//! The emulator renderer renders objects in a minecraft compatible manner.
//!
//! The [`EmulatorRenderer`] provides the necessary infrastructure for rendering but does not render
//! itself. Responsibilities includes management of long living resources such as static meshes /
//! textures and efficient uploading of short lived immediate objects used only inside one pass.
//! Rendering itself, is performed by [`EmulatorPipeline`] instances. This maximises flexibility of
//! the renderer.
//!
//! All rendering is done inside passes using a [`PassRecorder`]. Every pass uses a single
//! [`EmulatorPipeline`] to render its objects. Passes do not have to have a one to one
//! correspondence with frames. It is fully possible to use multiple passes and then combining the
//! output of each externally to form a frame. Or use passes asynchronously to the main render loop.
//! However currently b4d uses a single pass to render a single frame.

mod buffer;
mod worker;
mod global_objects;
mod pass;
mod memory;

pub mod pipeline;
pub mod debug_pipeline;
pub mod mc_shaders;
mod descriptors;

use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Mutex, Weak};
use std::sync::atomic::{AtomicU64, Ordering};
use ash::vk;

use crate::renderer::emulator::buffer::BufferPool;
use crate::renderer::emulator::worker::{run_worker, Share};
use crate::renderer::emulator::pipeline::EmulatorPipeline;

use crate::prelude::*;

pub use global_objects::StaticMeshId;

pub use pass::PassId;
pub use pass::PassRecorder;
pub use pass::ImmediateMeshId;
use crate::renderer::emulator::mc_shaders::{McUniform, Shader, ShaderId, VertexFormat};

pub struct EmulatorRenderer {
    id: UUID,
    weak: Weak<EmulatorRenderer>,
    device: Arc<DeviceContext>,
    worker: Arc<Share>,
    next_frame_id: AtomicU64,
    buffer_pool: Arc<Mutex<BufferPool>>,
    shader_database: Mutex<HashMap<ShaderId, Arc<Shader>>>,
}

impl EmulatorRenderer {
    pub(crate) fn new(device: Arc<DeviceContext>) -> Arc<Self> {
        let renderer = Arc::new_cyclic(|weak| {
            let pool = Arc::new(Mutex::new(BufferPool::new(device.clone())));

            Self {
                id: UUID::new(),
                weak: weak.clone(),
                device: device.clone(),
                worker: Arc::new(Share::new(device.clone(), pool.clone())),
                next_frame_id: AtomicU64::new(1),
                buffer_pool: pool,
                shader_database: Mutex::new(HashMap::new())
            }
        });

        let share = renderer.worker.clone();

        std::thread::spawn(move || {
            std::panic::catch_unwind(|| {
                run_worker(device, share);
            }).unwrap_or_else(|_| {
                log::error!("Emulator worker panicked!");
                std::process::exit(1);
            })
        });

        renderer
    }

    pub fn create_static_mesh(&self, data: &MeshData) -> StaticMeshId {
        self.worker.global_objects.create_static_mesh(data)
    }

    pub fn drop_static_mesh(&self, id: StaticMeshId) {
        self.worker.global_objects.mark_static_mesh(id)
    }

    pub fn create_shader(&self, vertex_format: &VertexFormat, used_uniforms: McUniform) -> ShaderId {
        let shader = Shader::new(*vertex_format, used_uniforms);
        let id = shader.get_id();

        let mut guard = self.shader_database.lock().unwrap();
        guard.insert(id, shader);

        id
    }

    pub fn drop_shader(&self, id: ShaderId) {
        let mut guard = self.shader_database.lock().unwrap();
        guard.remove(&id);
    }

    pub fn get_shader(&self, id: ShaderId) -> Option<Arc<Shader>> {
        let guard = self.shader_database.lock().unwrap();
        guard.get(&id).cloned()
    }

    pub fn start_pass(&self, pipeline: Arc<dyn EmulatorPipeline>) -> PassRecorder {
        let id = PassId::from_raw(self.next_frame_id.fetch_add(1, Ordering::SeqCst));
        PassRecorder::new(id, self.weak.upgrade().unwrap(), pipeline)
    }
}

impl PartialEq for EmulatorRenderer {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl Eq for EmulatorRenderer {
}

pub struct MeshData<'a> {
    pub vertex_data: &'a [u8],
    pub index_data: &'a [u8],
    pub vertex_stride: u32,
    pub index_count: u32,
    pub index_type: vk::IndexType,
    pub primitive_topology: vk::PrimitiveTopology,
}

impl<'a> MeshData<'a> {
    pub fn get_index_size(&self) -> u32 {
        match self.index_type {
            vk::IndexType::UINT8_EXT => 1u32,
            vk::IndexType::UINT16 => 2u32,
            vk::IndexType::UINT32 => 4u32,
            _ => {
                log::error!("Invalid index type");
                panic!()
            }
        }
    }
}

impl<'a> Debug for MeshData<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MeshData")
            .field("vertex_data.len()", &self.vertex_data.len())
            .field("index_data.len()", &self.index_data.len())
            .field("vertex_stride", &self.vertex_stride)
            .field("index_count", &self.index_count)
            .field("index_type", &self.index_type)
            .field("primitive_topology", &self.primitive_topology)
            .finish()
    }
}