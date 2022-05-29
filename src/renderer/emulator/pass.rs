use std::collections::HashMap;
use std::sync::Arc;

use ash::vk;

use crate::renderer::emulator::buffer::{BufferAllocation, BufferSubAllocator};
use crate::renderer::emulator::{EmulatorRenderer, MeshData, StaticMeshId};
use crate::renderer::emulator::worker::WorkerTask;
use crate::device::transfer::{BufferTransferRanges, StagingMemory};
use crate::objects::sync::SemaphoreOps;

use crate::prelude::*;
use crate::renderer::emulator::global_objects::StaticMeshDrawInfo;
use crate::renderer::emulator::pipeline::{DrawTask, EmulatorOutput, EmulatorPipeline, PipelineTask};

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

pub struct PassRecorder {
    id: PassId,
    renderer: Arc<EmulatorRenderer>,
    #[allow(unused)] // We just need to keep the pipeline alive
    pipeline: Arc<dyn EmulatorPipeline>,

    used_static_meshes: HashMap<StaticMeshId, StaticMeshDrawInfo>,

    current_buffer: Option<(BufferSubAllocator, StagingMemory)>,
    written_size: usize,
}

impl PassRecorder {
    pub(super) fn new(id: PassId, renderer: Arc<EmulatorRenderer>, pipeline: Arc<dyn EmulatorPipeline>) -> Self {
        renderer.worker.push_task(id, WorkerTask::StartPass(pipeline.start_pass()));

        Self {
            id,
            renderer,
            pipeline,

            used_static_meshes: HashMap::new(),

            current_buffer: None,
            written_size: 0,
        }
    }

    pub fn use_output(&mut self, output: Box<dyn EmulatorOutput + Send>) {
        self.renderer.worker.push_task(self.id, WorkerTask::UseOutput(output));
    }

    pub fn set_model_view_matrix(&mut self, matrix: &Mat4f32) {
        self.renderer.worker.push_task(self.id, WorkerTask::PipelineTask(PipelineTask::SetModelViewMatrix(*matrix)));
    }

    pub fn set_projection_matrix(&mut self, matrix: &Mat4f32) {
        self.renderer.worker.push_task(self.id, WorkerTask::PipelineTask(PipelineTask::SetProjectionMatrix(*matrix)));
    }

    pub fn draw_immediate(&mut self, data: &MeshData, type_id: u32) {
        let index_size = data.get_index_size();

        let vertex_format = self.renderer.get_vertex_format_info(data.vertex_format_id).unwrap_or_else(|| {
            log::error!("Invalid vertex format id {:?}", data.vertex_format_id);
            panic!()
        }).clone();

        let vertex_buffer = self.push_data(data.vertex_data, vertex_format.stride as u32);
        let index_buffer = self.push_data(data.index_data, index_size);

        let draw_task = DrawTask {
            vertex_buffer: vertex_buffer.buffer,
            index_buffer: index_buffer.buffer,
            vertex_offset: (vertex_buffer.offset / vertex_format.stride) as i32,
            first_index: (index_buffer.offset / (index_size as usize)) as u32,
            index_type: data.index_type,
            index_count: data.index_count,
            type_id,
        };
        self.renderer.worker.push_task(self.id, WorkerTask::PipelineTask(PipelineTask::Draw(draw_task)));
    }

    pub fn draw_static(&mut self, mesh_id: StaticMeshId, type_id: u32) {
        if !self.used_static_meshes.contains_key(&mesh_id) {
            let draw_info = self.renderer.worker.global_objects.inc_static_mesh(mesh_id);
            self.used_static_meshes.insert(mesh_id, draw_info);

            self.renderer.worker.push_task(self.id, WorkerTask::UseStaticMesh(mesh_id));
        }

        let draw_info = self.used_static_meshes.get(&mesh_id).unwrap();

        let draw_task = DrawTask {
            vertex_buffer: draw_info.buffer,
            index_buffer: draw_info.buffer,
            vertex_offset: 0,
            first_index: draw_info.first_index,
            index_type: draw_info.index_type,
            index_count: draw_info.index_count,
            type_id
        };

        self.renderer.worker.push_task(self.id, WorkerTask::PipelineTask(PipelineTask::Draw(draw_task)));
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

        let (buffer, size, wait_op) = self.renderer.buffer_pool.lock().unwrap().get_buffer(min_size);
        let op = self.renderer.device.get_transfer().prepare_buffer_acquire(buffer, None);
        self.renderer.device.get_transfer().acquire_buffer(op, SemaphoreOps::from_option(wait_op)).unwrap();

        self.renderer.worker.push_task(self.id, WorkerTask::UseDynamicBuffer(buffer));

        let allocator = BufferSubAllocator::new(buffer, size);
        let staging = self.renderer.device.get_transfer().request_staging_memory(allocator.get_buffer_size());

        self.current_buffer = Some((allocator, staging))
    }

    fn end_sub_allocator(&mut self) {
        if let Some((allocator, staging)) = self.current_buffer.take() {
            let transfer = self.renderer.device.get_transfer();
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
                self.renderer.worker.get_render_queue_family()
            )));
            let sync_id = transfer.release_buffer(op.clone()).unwrap();
            transfer.flush(sync_id);

            self.renderer.worker.push_task(self.id, WorkerTask::WaitTransferSync(sync_id))
        }
    }
}

impl Drop for PassRecorder {
    fn drop(&mut self) {
        self.end_sub_allocator();
        self.renderer.worker.push_task(self.id, WorkerTask::EndPass);
    }
}