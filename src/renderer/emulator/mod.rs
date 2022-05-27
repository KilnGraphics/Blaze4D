mod buffer;
mod worker;
mod static_mesh;

pub mod pipeline;
pub mod debug_pipeline;
pub mod pass;

use std::sync::{Arc, Mutex, Weak};
use std::sync::atomic::{AtomicU64, Ordering};
use ash::vk;
use gpu_allocator::d3d12::Allocation;

use crate::renderer::emulator::buffer::BufferPool;
use crate::renderer::emulator::pass::{PassId, PassRecorder};
use crate::renderer::emulator::worker::{run_worker, Share};

use crate::vk::DeviceEnvironment;

use crate::renderer::emulator::pipeline::EmulatorPipeline;
use crate::UUID;
use crate::vk::objects::buffer::Buffer;

pub use static_mesh::StaticMeshId;

pub(crate) struct EmulatorRenderer {
    id: UUID,
    weak: Weak<EmulatorRenderer>,
    device: DeviceEnvironment,
    worker: Arc<Share>,
    next_frame_id: AtomicU64,
    buffer_pool: Arc<Mutex<BufferPool>>,
    vertex_formats: Box<[VertexFormatInfo]>,
}

impl EmulatorRenderer {
    pub(crate) fn new(device: DeviceEnvironment, vertex_formats: VertexFormatSetBuilder) -> Arc<Self> {
        let renderer = Arc::new_cyclic(|weak| {
            let pool = Arc::new(Mutex::new(BufferPool::new(device.clone())));

            Self {
                id: UUID::new(),
                weak: weak.clone(),
                device: device.clone(),
                worker: Arc::new(Share::new(device.clone(), pool.clone())),
                next_frame_id: AtomicU64::new(1),
                buffer_pool: pool,
                vertex_formats: vertex_formats.formats.into_boxed_slice(),
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

    /// Lists all supported vertex formats.
    ///
    /// The index into the slice is the id of the format.
    pub fn get_vertex_formats(&self) -> &[VertexFormatInfo] {
        self.vertex_formats.as_ref()
    }

    /// Returns the vertex format info for some id.
    ///
    /// If the id is invalid [`None`] is returned.
    pub fn get_vertex_format(&self, id: VertexFormatId) -> Option<&VertexFormatInfo> {
        self.vertex_formats.get(id.get_raw() as usize)
    }

    pub fn create_static_mesh(&self, data: &MeshData) -> StaticMeshId {
        self.worker.create_static_mesh(data)
    }

    pub fn drop_static_mesh(&self, id: StaticMeshId) {
        self.worker.mark_static_mesh(id)
    }

    pub fn start_pass(&self, pipeline: Arc<dyn EmulatorPipeline>) -> PassRecorder {
        let id = PassId::from_raw(self.next_frame_id.fetch_add(1, Ordering::SeqCst));
        PassRecorder::new(id, self.weak.upgrade().unwrap(), pipeline)
    }
}

/// Information needed by the emulator renderer to process vertex data.
///
/// Individual pipelines may need additional information which is encoded in the pipeline type. See
/// [`EmulatorPipeline`] for more details.
#[derive(Copy, Clone, Debug)]
pub struct VertexFormatInfo {
    pub stride: usize,
}

pub struct VertexFormatSetBuilder {
    formats: Vec<VertexFormatInfo>,
}

impl VertexFormatSetBuilder {
    pub fn new() -> Self {
        Self {
            formats: Vec::new(),
        }
    }

    pub fn add_format(&mut self, format: VertexFormatInfo) -> VertexFormatId {
        let id = self.formats.len();
        self.formats.push(format);
        VertexFormatId::from_raw(id as u32)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct VertexFormatId(u32);

impl VertexFormatId {
    pub fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub fn get_raw(&self) -> u32 {
        self.0
    }
}

pub struct MeshData<'a> {
    pub vertex_data: &'a [u8],
    pub index_data: &'a [u8],
    pub index_count: u32,
    pub index_type: vk::IndexType,
    pub vertex_format_id: VertexFormatId,
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