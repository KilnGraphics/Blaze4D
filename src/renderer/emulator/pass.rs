use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use ash::vk;

use crate::renderer::emulator::immediate::{BufferAllocation, BufferSubAllocator};
use crate::renderer::emulator::{EmulatorRenderer, MeshData, StaticMeshId};
use crate::renderer::emulator::worker::WorkerTask;
use crate::device::transfer::{BufferTransferRanges, StagingMemory};
use crate::objects::sync::SemaphoreOps;

use crate::renderer::emulator::global_objects::StaticMeshDrawInfo;
use crate::renderer::emulator::mc_shaders::{DevUniform, McUniform, McUniformData, ShaderId};
use crate::renderer::emulator::pipeline::{DrawTask, EmulatorOutput, EmulatorPipeline, PipelineTask};
use crate::renderer::emulator::share::Share;
use crate::vk::objects::buffer::Buffer;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct PassId(u64);

impl PassId {
    pub fn from_raw(id: u64) -> Self {
        Self(id)
    }

    pub fn get_raw(&self) -> u64 {
        self.0
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct ImmediateMeshId(u32);

impl ImmediateMeshId {
    pub fn form_raw(id: u32) -> Self {
        Self(id)
    }

    pub fn get_raw(&self) -> u32 {
        self.0
    }
}

pub struct PassRecorder {
    id: PassId,
    share: Arc<Share>,

    used_shaders: HashSet<ShaderId>,
    used_static_meshes: HashMap<StaticMeshId, StaticMeshDrawInfo>,
    immediate_meshes: Vec<ImmediateMeshInfo>,

    current_buffer: Option<(BufferSubAllocator, StagingMemory)>,
    written_size: usize,

    #[allow(unused)] // We just need to keep the pipeline alive
    pipeline: Arc<dyn EmulatorPipeline>,
}

impl PassRecorder {
    pub(super) fn new(id: PassId, share: Arc<Share>, pipeline: Arc<dyn EmulatorPipeline>) -> Self {
        renderer.worker.push_task(id, WorkerTask::StartPass(pipeline.clone(), pipeline.start_pass()));

        Self {
            id,
            share,

            used_shaders: HashSet::new(),
            used_static_meshes: HashMap::new(),
            immediate_meshes: Vec::with_capacity(128),

            current_buffer: None,
            written_size: 0,

            pipeline,
        }
    }

    pub fn use_output(&mut self, output: Box<dyn EmulatorOutput + Send>) {
        self.share.worker.push_task(self.id, WorkerTask::UseOutput(output));
    }

    pub fn update_uniform(&mut self, data: &McUniformData, shader: ShaderId) {
        self.use_shader(shader);
        self.share.worker.push_task(self.id, WorkerTask::PipelineTask(PipelineTask::UpdateUniform(shader, *data)))
    }

    pub fn upload_immediate(&mut self, data: &MeshData) -> ImmediateMeshId {
        let index_size = data.get_index_size();

        let vertex_buffer = self.push_data(data.vertex_data, data.vertex_stride);
        let index_buffer = self.push_data(data.index_data, index_size);

        let id = self.immediate_meshes.len() as u32;
        self.immediate_meshes.push(ImmediateMeshInfo {
            vertex_buffer: vertex_buffer.buffer,
            index_buffer: index_buffer.buffer,
            vertex_offset: (vertex_buffer.offset / (data.vertex_stride as usize)) as i32,
            first_index: (index_buffer.offset / (index_size as usize)) as u32,
            index_type: data.index_type,
            index_count: data.index_count,
            primitive_topology: data.primitive_topology
        });

        ImmediateMeshId::form_raw(id)
    }

    pub fn draw_immediate(&mut self, id: ImmediateMeshId, shader: ShaderId, depth_write_enable: bool) {
        self.use_shader(shader);

        let mesh_data = self.immediate_meshes.get(id.get_raw() as usize).unwrap();

        let draw_task = DrawTask {
            vertex_buffer: mesh_data.vertex_buffer,
            index_buffer: mesh_data.index_buffer,
            vertex_offset: mesh_data.vertex_offset,
            first_index: mesh_data.first_index,
            index_type: mesh_data.index_type,
            index_count: mesh_data.index_count,
            shader,
            primitive_topology: mesh_data.primitive_topology,
            depth_write_enable,
        };
        self.share.worker.push_task(self.id, WorkerTask::PipelineTask(PipelineTask::Draw(draw_task)));
    }

    pub fn draw_static(&mut self, mesh_id: StaticMeshId, shader: ShaderId) {
        self.use_shader(shader);

        if !self.used_static_meshes.contains_key(&mesh_id) {
            let draw_info = self.share.worker.global_objects.inc_static_mesh(mesh_id);
            self.used_static_meshes.insert(mesh_id, draw_info);

            self.share.worker.push_task(self.id, WorkerTask::UseStaticMesh(mesh_id));
        }

        let draw_info = self.used_static_meshes.get(&mesh_id).unwrap();

        let draw_task = DrawTask {
            vertex_buffer: draw_info.buffer,
            index_buffer: draw_info.buffer,
            vertex_offset: 0,
            first_index: draw_info.first_index,
            index_type: draw_info.index_type,
            index_count: draw_info.index_count,
            shader,
            primitive_topology: draw_info.primitive_topology,
            depth_write_enable: false,
        };

        self.share.worker.push_task(self.id, WorkerTask::PipelineTask(PipelineTask::Draw(draw_task)));
    }

    fn use_shader(&mut self, shader: ShaderId) {
        if self.used_shaders.insert(shader) {
            self.pipeline.inc_shader_used(shader);
            self.share.worker.push_task(self.id, WorkerTask::UseShader(shader));
        }
    }

    fn push_data(&mut self, data: &[u8], alignment: u32) -> BufferAllocation {
        let alloc = self.allocate_memory(data.len(), alignment);
        let (_, staging) = self.current_buffer.as_ref().unwrap();

        unsafe {
            staging.write_offset(data, alloc.offset);
        }
        self.written_size = alloc.offset + data.len();

        alloc
    }

    fn allocate_memory(&mut self, size: usize, alignment: u32) -> BufferAllocation {
        if self.current_buffer.is_none() {
            self.new_sub_allocator(size);
        }

        match self.current_buffer.as_mut().unwrap().0.allocate(size, alignment) {
            None => {
                self.new_sub_allocator(size);
                self.current_buffer.as_mut().unwrap().0.allocate(size, alignment).unwrap()
            }
            Some(alloc) => {
                alloc
            }
        }
    }

    fn new_sub_allocator(&mut self, min_size: usize) {
        self.end_sub_allocator();

        let (buffer, size, wait_op) = self.share.buffer_pool.lock().unwrap().get_buffer(min_size);
        let op = self.share.device.get_transfer().prepare_buffer_acquire(buffer, None);
        self.share.device.get_transfer().acquire_buffer(op, SemaphoreOps::from_option(wait_op)).unwrap();

        self.share.worker.push_task(self.id, WorkerTask::UseDynamicBuffer(buffer));

        let allocator = BufferSubAllocator::new(buffer, size);
        let staging = self.share.device.get_transfer().request_staging_memory(allocator.get_buffer_size());

        self.current_buffer = Some((allocator, staging))
    }

    fn end_sub_allocator(&mut self) {
        if let Some((allocator, staging)) = self.current_buffer.take() {
            let transfer = self.share.device.get_transfer();
            let buffer = allocator.get_buffer();

            unsafe {
                staging.copy_to_buffer(buffer, BufferTransferRanges::new_single(
                    0,
                    0,
                    self.written_size as vk::DeviceSize
                ));
            }
            self.written_size = 0;

            let op = transfer.prepare_buffer_release(buffer, Some((
                vk::PipelineStageFlags2::VERTEX_INPUT | vk::PipelineStageFlags2::INDEX_INPUT,
                vk::AccessFlags2::VERTEX_ATTRIBUTE_READ | vk::AccessFlags2::INDEX_READ,
                self.share.worker.get_render_queue_family()
            )));
            let sync_id = transfer.release_buffer(op.clone()).unwrap();
            transfer.flush(sync_id);

            self.share.worker.push_task(self.id, WorkerTask::WaitTransferSync(sync_id))
        }
    }
}

impl Drop for PassRecorder {
    fn drop(&mut self) {
        self.end_sub_allocator();
        self.share.worker.push_task(self.id, WorkerTask::EndPass);
    }
}

struct ImmediateMeshInfo {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    vertex_offset: i32,
    first_index: u32,
    index_type: vk::IndexType,
    index_count: u32,
    primitive_topology: vk::PrimitiveTopology,
}