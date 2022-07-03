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

mod immediate;
mod worker;
mod global_objects;
mod pass;

pub mod pipeline;
pub mod debug_pipeline;
pub mod mc_shaders;
mod descriptors;
mod share;
mod staging;

use std::fmt::{Debug, Formatter};
use std::panic::RefUnwindSafe;
use std::sync::Arc;
use ash::vk;

use crate::renderer::emulator::worker::run_worker;
use crate::renderer::emulator::pipeline::EmulatorPipeline;

use crate::prelude::*;

pub use global_objects::{GlobalMesh, GlobalImage, ImageData, SamplerInfo};

pub use pass::PassId;
pub use pass::PassRecorder;
pub use pass::ImmediateMeshId;

use share::Share;
use crate::renderer::emulator::mc_shaders::{McUniform, Shader, ShaderId, VertexFormat};
use crate::util::format::Format;

pub struct EmulatorRenderer {
    share: Arc<Share>,
    placeholder_image: Arc<GlobalImage>,
    placeholder_sampler: SamplerInfo,
    worker: std::thread::JoinHandle<()>,
}

impl EmulatorRenderer {
    pub(crate) fn new(device: Arc<DeviceContext>) -> Self {
        let share = Arc::new(Share::new(device.clone()));

        let share2 = share.clone();
        let worker = std::thread::spawn(move || {
            std::panic::catch_unwind(|| {
                run_worker(device,share2);
            }).unwrap_or_else(|_| {
                log::error!("Emulator worker panicked!");
                std::process::exit(1);
            })
        });

        let placeholder_image = Self::create_placeholder_image(share.clone());
        let placeholder_sampler = SamplerInfo {
            mag_filter: vk::Filter::LINEAR,
            min_filter: vk::Filter::LINEAR,
            mipmap_mode: vk::SamplerMipmapMode::LINEAR,
            address_mode_u: vk::SamplerAddressMode::REPEAT,
            address_mode_v: vk::SamplerAddressMode::REPEAT,
            anisotropy_enable: false
        };

        Self {
            share,
            placeholder_image,
            placeholder_sampler,
            worker,
        }
    }

    pub fn get_device(&self) -> &Arc<DeviceContext> {
        self.share.get_device()
    }

    pub fn create_global_mesh(&self, data: &MeshData) -> Arc<GlobalMesh> {
        GlobalMesh::new(self.share.clone(), data).unwrap()
    }

    pub fn create_global_image(&self, size: Vec2u32, format: &'static Format) -> Arc<GlobalImage> {
        GlobalImage::new(self.share.clone(), size, 1, format).unwrap()
    }

    pub fn create_global_image_mips(&self, size: Vec2u32, mip_levels: u32, format: &'static Format) -> Arc<GlobalImage> {
        GlobalImage::new(self.share.clone(), size, mip_levels, format).unwrap()
    }

    pub fn create_shader(&self, vertex_format: &VertexFormat, used_uniforms: McUniform) -> ShaderId {
        self.share.create_shader(vertex_format, used_uniforms)
    }

    pub fn drop_shader(&self, id: ShaderId) {
        self.share.drop_shader(id)
    }

    pub fn get_shader(&self, id: ShaderId) -> Option<Arc<Shader>> {
        self.share.get_shader(id)
    }

    pub fn start_pass(&self, pipeline: Arc<dyn EmulatorPipeline>) -> PassRecorder {
        PassRecorder::new(self.share.clone(), pipeline, self.placeholder_image.clone(), &self.placeholder_sampler)
    }

    fn create_placeholder_image(share: Arc<Share>) -> Arc<GlobalImage> {
        let size = Vec2u32::new(256, 256);

        let mut data: Box<[_]> = std::iter::repeat([0u8, 0u8, 0u8, 255u8]).take((size[0] as usize) * (size[1] as usize)).collect();
        for x in 0..(size[0] as usize) {
            for y in 0..(size[1] as usize) {
                if ((x / 128) + (y / 128)) % 2 == 0 {
                    data[(y * (size[0] as usize)) + x] = [255u8, 0u8, 255u8, 255u8];
                }
            }
        }

        let bytes = data.as_ref().as_bytes();

        let info = ImageData {
            data: bytes,
            row_stride: 0,
            offset: Vec2u32::new(0, 0),
            extent: size
        };

        let image = GlobalImage::new(share, size, 1, &Format::R8G8B8A8_SRGB).unwrap();
        image.update_regions(std::slice::from_ref(&info));
        image
    }
}

impl PartialEq for EmulatorRenderer {
    fn eq(&self, other: &Self) -> bool {
        self.share.eq(&other.share)
    }
}

impl Eq for EmulatorRenderer {
}

impl RefUnwindSafe for EmulatorRenderer { // Join handle is making issues
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
