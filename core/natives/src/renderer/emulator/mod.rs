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
mod program;
mod c_api;
mod objects;

use std::any::Any;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::panic::RefUnwindSafe;
use std::pin::Pin;
use std::ptr::NonNull;
use std::sync::{Arc, Weak};
use std::thread::JoinHandle;
use ash::vk;
use bumpalo::Bump;
use bytemuck::cast_slice;
use higher_order_closure::higher_order_closure;

use crate::renderer::emulator::worker::{CopyBufferToStagingTask, CopyImageToStagingTask, CopyStagingToBufferTask, CopyStagingToImageTask, DrawTask, EmulatorTaskContainer, run_worker, WorkerTask2};
use crate::renderer::emulator::pipeline::EmulatorPipeline;

use crate::prelude::*;

pub use global_objects::{GlobalMesh, GlobalImage, ImageData, SamplerInfo};

pub use pass::PassId;
pub use pass::PassRecorder;
pub use pass::ImmediateMeshId;

use share::Share;
use crate::allocator::Allocation;
use crate::define_uuid_type;
use crate::objects::sync::SemaphoreOp;
use crate::renderer::emulator::mc_shaders::{McUniform, Shader, ShaderId, VertexFormat};
use crate::renderer::emulator::share::{Share2};
use crate::renderer::emulator::staging::StagingAllocationId2;
use crate::util::format::Format;

pub use objects::{Buffer, BufferInfo, BufferId, Image, ImageInfo, ImageSize, ImageId, GraphicsPipeline};

pub type BufferArc = Arc<Buffer>;
pub type ImageArc = Arc<Image>;

pub struct Emulator2 {
    share: Arc<Share2>,
    worker: Option<JoinHandle<()>>,
}

impl Emulator2 {
    pub fn new(device: Arc<DeviceContext>) -> Self {
        let queue = device.get_main_queue().clone();
        let (share, worker) = Share2::new(device, queue);

        Self {
            share,
            worker: Some(worker),
        }
    }

    pub fn create_persistent_buffer(&self, size: u64) -> BufferArc {
        Arc::new(Buffer::new_persistent(self.share.clone(), size))
    }

    pub fn create_persistent_color_image(&self, format: vk::Format, size: ImageSize) -> ImageArc {
        Arc::new(Image::new_persistent_color(self.share.clone(), format, size))
    }

    pub fn cmd_write_buffer(&self, buffer: BufferArc, offset: u64, data: &[u8]) {
        // TODO usize to u64 cast may not be safe
        let (memory, alloc) = self.share.allocate_staging(data.len() as u64, 1);
        unsafe {
            std::slice::from_raw_parts_mut(memory.mapped_memory.as_ptr(), data.len()).copy_from_slice(data)
        };

        self.submit_cmd(higher_order_closure!{ for<'a> |b: TaskBuilder<'a>| -> Result<CopyStagingToBuffer<'a>, ()> {
            let mut b = b;

            let region = vk::BufferCopy {
                src_offset: memory.buffer_offset,
                dst_offset: offset,
                size: data.len() as vk::DeviceSize,
            };

            Ok(CopyStagingToBuffer {
                staging_allocation: alloc,
                staging_buffer: memory.buffer,
                dst_buffer: buffer,
                copy_regions: b.alloc([region]).into()
            })
        }}).unwrap();
    }

    pub fn cmd_read_buffer<'a>(&self, buffer: BufferArc, offset: u64, dst: &'a mut [u8]) -> ReadToken<'a> {
        // TODO usize to u64 cast may not be safe
        let (memory, alloc) = self.share.allocate_staging(dst.len() as u64, 1);

        let id = self.submit_cmd(higher_order_closure!{ for<'a> |b: TaskBuilder<'a>| -> Result<CopyBufferToStaging<'a>, ()> {
            let mut b = b;

            let region = vk::BufferCopy {
                src_offset: offset,
                dst_offset: memory.buffer_offset,
                size: dst.len() as vk::DeviceSize,
            };

            Ok(CopyBufferToStaging {
                staging_buffer: memory.buffer,
                src_buffer: buffer,
                copy_regions: b.alloc([region]).into()
            })
        }}).unwrap();

        ReadToken::new(self.share.clone(), id, vec![ReadCopy {
            dst,
            staging_memory: memory.mapped_memory,
            staging_allocation: alloc
        }])
    }

    pub fn cmd_write_sub_image(&self, image: ImageArc, copy_region: &vk::BufferImageCopy, data: &[u8]) {
        // TODO usize to u64 cast may not be safe
        let (memory, alloc) = self.share.allocate_staging(data.len() as u64, 1);
        unsafe {
            std::slice::from_raw_parts_mut(memory.mapped_memory.as_ptr(), data.len()).copy_from_slice(data)
        };

        self.submit_cmd(higher_order_closure!{ for<'a> |b: TaskBuilder<'a>| -> Result<CopyStagingToImage<'a>, ()> {
            let mut b = b;

            let mut copy_region = *copy_region;
            copy_region.buffer_offset += memory.buffer_offset;

            Ok(CopyStagingToImage {
                staging_allocation: alloc,
                staging_buffer: memory.buffer,
                dst_image: image,
                copy_regions: b.alloc([copy_region]).into()
            })
        }}).unwrap();
    }

    pub fn cmd_read_sub_image<'a>(&self, image: ImageArc, copy_region: &vk::BufferImageCopy, dst: &'a mut [u8]) -> ReadToken<'a> {
        let (memory, alloc) = self.share.allocate_staging(dst.len() as u64, 1);
        unsafe {
            // Need to do this because not all parts of the buffer are guaranteed to be written to by the copy regions
            std::slice::from_raw_parts_mut(memory.mapped_memory.as_ptr(), dst.len()).copy_from_slice(dst)
        };

        let id = self.submit_cmd(higher_order_closure!{ for<'a> |b: TaskBuilder<'a>| -> Result<CopyImageToStaging<'a>, ()> {
            let mut b = b;

            let mut copy_region = *copy_region;
            copy_region.buffer_offset += memory.buffer_offset;

            Ok(CopyImageToStaging {
                staging_buffer: memory.buffer,
                src_image: image,
                copy_regions: b.alloc([copy_region]).into()
            })
        }}).unwrap();

        ReadToken::new(self.share.clone(), id, vec![ReadCopy {
            dst,
            staging_memory: memory.mapped_memory,
            staging_allocation: alloc
        }])
    }

    pub fn cmd<F, R>(&self, f: F) -> Result<(), R>
        where F: for<'a> TaskBuilderFn<'a, Err=R> {

        self.submit_cmd(f)?;
        Ok(())
    }

    fn submit_cmd<F, R>(&self, f: F) -> Result<u64, R>
        where F: for<'a> TaskBuilderFn<'a, Err=R> {

        let alloc = Bump::new();

        let task = f(TaskBuilder {
            alloc: &alloc,
        })?;
        let task = task.into_task(&alloc);

        let task = unsafe {
            // Need to transmute to allow moving to heap
            let transmuted: EmulatorTask<'static> = std::mem::transmute(task);
            EmulatorTaskContainer::new(alloc, transmuted)
        };

        Ok(self.share.push_task(task))
    }

    pub fn create_export_set(&self) -> ExportSet {
        todo!()
    }

    pub fn flush(&self) {
        self.share.flush();
    }
}

pub trait TaskBuilderFn<'a> where Self: FnOnce(TaskBuilder<'a>) -> Result<Self::Ok, Self::Err> {
    type Ok: IntoEmulatorTask<'a>;
    type Err;
}

impl<'a, F, Ok, Err> TaskBuilderFn<'a> for F
    where Self: FnOnce(TaskBuilder<'a>) -> Result<Ok, Err>, Ok: IntoEmulatorTask<'a> {

    type Ok = Ok;
    type Err = Err;
}

impl Drop for Emulator2 {
    fn drop(&mut self) {
        self.share.shutdown();
        // The error state is already handled by the share and cleanup
        let _ = self.worker.take().unwrap().join();
        self.share.cleanup();
    }
}

pub type BBox<'a, T> = bumpalo::boxed::Box<'a, T>;

pub struct TaskBuilder<'a> {
    alloc: &'a Bump,
}

impl<'a> TaskBuilder<'a> {
    pub fn allocator(&self) -> &'a Bump {
        self.alloc
    }

    pub fn alloc<O: Sized>(&self, data: O) -> BBox<'a, O> {
        BBox::new_in(data, self.alloc)
    }
}

pub trait IntoEmulatorTask<'a> {
    fn into_task(self, alloc: &'a Bump) -> EmulatorTask<'a>;
}

pub enum EmulatorTask<'a> {
    CopyStagingToBuffer(BBox<'a, CopyStagingToBuffer<'a>>),
    CopyBufferToStaging(BBox<'a, CopyBufferToStaging<'a>>),
    CopyStagingToImage(BBox<'a, CopyStagingToImage<'a>>),
    CopyImageToStaging(BBox<'a, CopyImageToStaging<'a>>),
    CopyBuffer(BBox<'a, CopyBufferTask<'a>>),
    CopyBufferToImage(BBox<'a, CopyBufferToImage<'a>>),
    CopyImageToBuffer(BBox<'a, CopyImageToBuffer<'a>>),
}

impl<'a> EmulatorTask<'a> {
    fn extract_objects(&self, objects: &mut Vec<Arc<dyn Any>>) {
        match self {
            EmulatorTask::CopyStagingToBuffer(task) => {
                objects.reserve(1);
                objects.push(task.dst_buffer.clone());
            }
            EmulatorTask::CopyBufferToStaging(task) => {
                objects.reserve(1);
                objects.push(task.src_buffer.clone());
            }
            EmulatorTask::CopyStagingToImage(task) => {
                objects.reserve(1);
                objects.push(task.dst_image.clone());
            }
            EmulatorTask::CopyImageToStaging(task) => {
                objects.reserve(1);
                objects.push(task.src_image.clone());
            }
            EmulatorTask::CopyBuffer(task) => {
                objects.reserve(2);
                objects.push(task.src_buffer.clone());
                objects.push(task.dst_buffer.clone());
            }
            EmulatorTask::CopyBufferToImage(task) => {
                objects.reserve(2);
                objects.push(task.src_buffer.clone());
                objects.push(task.dst_image.clone());
            }
            EmulatorTask::CopyImageToBuffer(task) => {
                objects.reserve(2);
                objects.push(task.src_image.clone());
                objects.push(task.dst_buffer.clone());
            }
        }
    }
}

pub struct CopyStagingToBuffer<'a> {
    staging_allocation: StagingAllocationId2,
    staging_buffer: vk::Buffer,
    dst_buffer: BufferArc,
    copy_regions: BBox<'a, [vk::BufferCopy]>,
}

impl<'a> IntoEmulatorTask<'a> for CopyStagingToBuffer<'a> {
    fn into_task(self, alloc: &'a Bump) -> EmulatorTask<'a> {
        EmulatorTask::CopyStagingToBuffer(BBox::new_in(self, alloc))
    }
}

pub struct CopyBufferToStaging<'a> {
    staging_buffer: vk::Buffer,
    src_buffer: BufferArc,
    copy_regions: BBox<'a, [vk::BufferCopy]>,
}

impl<'a> IntoEmulatorTask<'a> for CopyBufferToStaging<'a> {
    fn into_task(self, alloc: &'a Bump) -> EmulatorTask<'a> {
        EmulatorTask::CopyBufferToStaging(BBox::new_in(self, alloc))
    }
}

pub struct CopyBufferTask<'a> {
    pub src_buffer: BufferArc,
    pub dst_buffer: BufferArc,
    pub regions: bumpalo::boxed::Box<'a, [vk::BufferCopy]>,
}

impl<'a> IntoEmulatorTask<'a> for CopyBufferTask<'a> {
    fn into_task(self, alloc: &'a Bump) -> EmulatorTask<'a> {
        EmulatorTask::CopyBuffer(BBox::new_in(self, alloc))
    }
}

pub struct CopyStagingToImage<'a> {
    staging_allocation: StagingAllocationId2,
    staging_buffer: vk::Buffer,
    dst_image: ImageArc,
    copy_regions: BBox<'a, [vk::BufferImageCopy]>,
}

impl<'a> IntoEmulatorTask<'a> for CopyStagingToImage<'a> {
    fn into_task(self, alloc: &'a Bump) -> EmulatorTask<'a> {
        EmulatorTask::CopyStagingToImage(BBox::new_in(self, alloc))
    }
}

pub struct CopyImageToStaging<'a> {
    staging_buffer: vk::Buffer,
    src_image: ImageArc,
    copy_regions: BBox<'a, [vk::BufferImageCopy]>,
}

impl<'a> IntoEmulatorTask<'a> for CopyImageToStaging<'a> {
    fn into_task(self, alloc: &'a Bump) -> EmulatorTask<'a> {
        EmulatorTask::CopyImageToStaging(BBox::new_in(self, alloc))
    }
}

pub struct CopyBufferToImage<'a> {
    pub src_buffer: BufferArc,
    pub dst_image: ImageArc,
    pub copy_regions: BBox<'a, [vk::BufferImageCopy]>,
}

impl<'a> IntoEmulatorTask<'a> for CopyBufferToImage<'a> {
    fn into_task(self, alloc: &'a Bump) -> EmulatorTask<'a> {
        EmulatorTask::CopyBufferToImage(BBox::new_in(self, alloc))
    }
}

pub struct CopyImageToBuffer<'a> {
    pub src_image: ImageArc,
    pub dst_buffer: BufferArc,
    pub copy_regions: BBox<'a, [vk::BufferImageCopy]>,
}

impl<'a> IntoEmulatorTask<'a> for CopyImageToBuffer<'a> {
    fn into_task(self, alloc: &'a Bump) -> EmulatorTask<'a> {
        EmulatorTask::CopyImageToBuffer(BBox::new_in(self, alloc))
    }
}

/// Description of a emulator pipeline.
pub struct PipelineCreateInfo<'a> {
    /// A list of shader module code used to create shaders.
    pub shader_modules: &'a [&'a [u32]],

    /// Description of the vertex shader stage.
    pub vertex_shader: PipelineShaderInfo<'a>,

    /// Description of the fragment shader stage.
    pub fragment_shader: PipelineShaderInfo<'a>,

    /// A list of all input attribute locations used by the pipeline.
    pub input_attributes: &'a [u32],
}

/// Description of a emulator pipeline shader stage.
pub struct PipelineShaderInfo<'a> {
    /// The index of the module in [`PipelineCreateInfo::shader_modules`] used for this stage.
    pub index: usize,

    /// The name of the entry point for this stage.
    pub entry: &'a str,

    /// Optional specialization info used when creating the shader stage.
    pub specialization_info: Option<&'a vk::SpecializationInfo>,
}

#[derive(Clone)]
pub struct PipelineState {
    primitive_topology: vk::PrimitiveTopology,
    polygon_mode: vk::PolygonMode,
    cull_mode: vk::CullModeFlags,
    front_face: vk::FrontFace,
    line_width: f32,
    depth_test: Option<vk::CompareOp>,
    depth_write_enable: bool,
    stencil_test: Option<(vk::StencilOpState, vk::StencilOpState)>,
    color_blend: Box<[Option<PipelineColorBlendState>]>,
    viewport: (Vec2f32, Vec2f32),
    scissor: (Vec2u32, Vec2u32),
}

#[derive(Copy, Clone, PartialEq, Hash, Debug)]
pub struct PipelineColorBlendState {
    src_color_blend_factor: vk::BlendFactor,
    dst_color_blend_factor: vk::BlendFactor,
    color_blend_op: vk::BlendOp,
    src_alpha_blend_factor: vk::BlendFactor,
    dst_alpha_blend_factor: vk::BlendFactor,
    alpha_blend_op: vk::BlendOp,
}

#[derive(Clone)]
pub struct PipelineInputAttribute {
    stride: u32,
    format: vk::Format,
    input_rate: vk::VertexInputRate,
}

#[derive(Clone, Eq, PartialEq)]
pub struct FramebufferState {
    render_area: (Vec2u32, Vec2u32),
    color_attachments: Box<[Arc<Image>]>,
    depth_attachment: Option<Arc<Image>>,
    stencil_attachment: Option<Arc<Image>>,
}

pub struct ExportSet {
    emulator: Arc<Share2>,
    images: Box<[Arc<Image>]>,
    image_infos: Box<[ExportImageInfo]>
}

impl ExportSet {
    pub fn get_images(&self) -> &[ExportImageInfo] {
        &self.image_infos
    }

    pub fn export(&self) -> ExportHandle {
        todo!()
    }
}

pub struct ExportImageInfo {
    image: vk::Image,
    size: ImageSize,
    format: vk::Format,
}

impl ExportImageInfo {
    pub unsafe fn get_image(&self) -> vk::Image {
        self.image
    }

    pub fn get_size(&self) -> ImageSize {
        self.size
    }

    pub fn get_format(&self) -> vk::Format {
        self.format
    }
}

pub struct ExportHandle {
    emulator: Arc<Share2>,
    wait_op: SemaphoreOp,
}

impl ExportHandle {
    pub unsafe fn get_wait_op(&self) -> SemaphoreOp {
        self.wait_op
    }
}

impl Drop for ExportHandle {
    fn drop(&mut self) {
        todo!()
    }
}

pub struct ReadToken<'a> {
    payload: ReadTokenInternal<'a>,
}

impl<'a> ReadToken<'a> {
    fn new(share: Arc<Share2>, wait_value: u64, copies: Vec<ReadCopy<'a>>) -> Self {
        Self {
            payload: ReadTokenInternal {
                share: Some(share),
                wait_value,
                copies,
            }
        }
    }

    pub fn join(&mut self, mut other: ReadToken<'a>) {
        self.payload.join(&mut other.payload);
    }

    pub fn await_ready(mut self) {
        self.payload.await_ready();
    }
}

/// Internal container for the [`ReadToken`]. Necessary because we cannot create functions that
/// consume self if the struct implements drop.
struct ReadTokenInternal<'a> {
    share: Option<Arc<Share2>>,
    wait_value: u64,
    copies: Vec<ReadCopy<'a>>,
}

impl<'a> ReadTokenInternal<'a> {
    fn join(&mut self, other: &mut ReadTokenInternal<'a>) {
        if self.share.is_none() {
            panic!("Called join on already awaited read token");
        }
        if self.share != other.share {
            panic!("Called join on read tokens with non equal share");
        }

        self.wait_value = std::cmp::max(self.wait_value, other.wait_value);
        self.copies.extend(std::mem::replace(&mut other.copies, Vec::new()).into_iter());
        other.share = None;
    }

    fn await_ready(&mut self) {
        if let Some(share) = self.share.take() {
            share.flush();
            share.wait_for_task(self.wait_value);

            for copy in &mut self.copies {
                unsafe { copy.exec_copy() };
            }
            unsafe {
                share.free_staging(std::mem::replace(&mut self.copies, Vec::new()).into_iter().map(|c| c.staging_allocation));
            }
        }
    }
}

impl<'a> Drop for ReadTokenInternal<'a> {
    fn drop(&mut self) {
        self.await_ready()
    }
}

struct ReadCopy<'a> {
    dst: &'a mut [u8],
    staging_memory: NonNull<u8>,
    staging_allocation: StagingAllocationId2,
}

impl<'a> ReadCopy<'a> {
    unsafe fn exec_copy(&mut self) {
        self.dst.copy_from_slice(std::slice::from_raw_parts(self.staging_memory.as_ptr(), self.dst.len()));
    }
}













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

        let bytes = cast_slice(data.as_ref());

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



#[cfg(test)]
mod test {
    use ash::vk;
    use rand::{RngCore, SeedableRng};
    use crate::prelude::Vec2u32;
    use crate::renderer::emulator::{Emulator2, ImageSize};

    #[test]
    fn startup_shutdown() {
        crate::init_test_env();
        let (_, device) = crate::test::create_test_instance_device(None).unwrap();

        let emulator = Emulator2::new(device.clone());
        drop(emulator);
    }

    #[test]
    fn buffer_copies() {
        crate::init_test_env();
        let (_, device) = crate::test::create_test_instance_device(None).unwrap();
        let emulator = Emulator2::new(device);

        let buffer = emulator.create_persistent_buffer(1024*1024);

        let sizes_offsets = &[
            (1usize, 0usize),
            (1, 242),
            (1, (1024*1024) - 1),
            (2, 0),
            (2, 234389),
            (2, (1024*1024) - 2),
            (16, 0),
            (16, 34893),
            (16, (1024*1024) - 16),
            (242, 0),
            (242, 898222),
            (242, (1024*1024) - 242),
            (3489, 0),
            (3489, 2329),
            (3489, (1024*1024) - 3489),
            (324892, 0),
            (324892, 1),
            (324892, (1024*1024) - 324892),
            (1024*1024, 0)
        ];
        let mut rand = rand::rngs::StdRng::seed_from_u64(0x55C18F5FA3B21BF6u64);
        for (size, offset) in sizes_offsets {
            let mut data = Vec::new();
            data.resize(*size, 0u8);
            rand.fill_bytes(&mut data);

            let mut dst = Vec::new();
            dst.resize(*size, 0u8);

            emulator.cmd_write_buffer(buffer.clone(), *offset as u64, &data);
            emulator.cmd_read_buffer(buffer.clone(), *offset as u64, &mut dst).await_ready();

            assert_eq!(data, dst);
        }
    }

    #[test]
    fn image_copies() {
        crate::init_test_env();
        let (_, device) = crate::test::create_test_instance_device(None).unwrap();
        let emulator = Emulator2::new(device);

        let image = emulator.create_persistent_color_image(vk::Format::R8G8B8A8_SRGB, ImageSize::new_2d(Vec2u32::new(1024, 1024), 1, 1));

        let subresource = vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            mip_level: 0,
            base_array_layer: 0,
            layer_count: 1
        };
        let sizes = &[
            vk::BufferImageCopy {
                buffer_offset: 0,
                buffer_row_length: 0,
                buffer_image_height: 0,
                image_subresource: subresource.clone(),
                image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
                image_extent: vk::Extent3D { width: 1, height: 1, depth: 1 },
            },
            vk::BufferImageCopy {
                buffer_offset: 0,
                buffer_row_length: 0,
                buffer_image_height: 0,
                image_subresource: subresource.clone(),
                image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
                image_extent: vk::Extent3D { width: 234, height: 34, depth: 1 },
            },
            vk::BufferImageCopy {
                buffer_offset: 0,
                buffer_row_length: 0,
                buffer_image_height: 0,
                image_subresource: subresource.clone(),
                image_offset: vk::Offset3D { x: 236, y: 12, z: 0 },
                image_extent: vk::Extent3D { width: 102, height: 936, depth: 1 },
            },
            vk::BufferImageCopy {
                buffer_offset: 0,
                buffer_row_length: 0,
                buffer_image_height: 0,
                image_subresource: subresource.clone(),
                image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
                image_extent: vk::Extent3D { width: 1024, height: 1024, depth: 1 },
            },
        ];
        let mut rand = rand::rngs::StdRng::seed_from_u64(0x55C18F5FA3B21BF6u64);
        for copy in sizes {
            let bytes = (copy.image_extent.width as usize) * (copy.image_extent.height as usize) * 4;
            let mut data = Vec::new();
            data.resize(bytes, 0u8);
            rand.fill_bytes(&mut data);

            let mut dst = Vec::new();
            dst.resize(bytes, 0u8);

            emulator.cmd_write_sub_image(image.clone(), &copy, &data);
            emulator.cmd_read_sub_image(image.clone(), &copy, &mut dst).await_ready();

            assert_eq!(data, dst);
        }
    }
}