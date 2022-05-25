use std::sync::Arc;

use ash::vk;

use crate::renderer::emulator::buffer::{BufferAllocation, BufferSubAllocator};
use crate::renderer::emulator::EmulatorRenderer;
use crate::renderer::emulator::worker::WorkerTask;
use crate::device::transfer::BufferTransferRanges;
use crate::objects::sync::SemaphoreOps;

use crate::prelude::*;
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

pub struct ObjectData<'a> {
    pub vertex_data: &'a [u8],
    pub index_data: &'a [u8],
    pub index_count: u32,
    pub type_id: u32,
}

pub struct PassRecorder {
    id: PassId,
    renderer: Arc<EmulatorRenderer>,
    pipeline: Arc<dyn EmulatorPipeline>,
    current_buffer: Option<BufferSubAllocator>,
}

impl PassRecorder {
    pub(super) fn new(id: PassId, renderer: Arc<EmulatorRenderer>, pipeline: Arc<dyn EmulatorPipeline>) -> Self {
        renderer.worker.push_task(id, WorkerTask::StartPass(pipeline.start_pass()));

        Self {
            id,
            renderer,
            pipeline,
            current_buffer: None,
        }
    }

    pub fn use_output(&mut self, output: Box<dyn EmulatorOutput + Send>) {
        self.renderer.worker.push_task(self.id, WorkerTask::UseOutput(output));
    }

    pub fn set_model_view_matrix(&mut self, matrix: Mat4f32) {
        self.renderer.worker.push_task(self.id, WorkerTask::PipelineTask(PipelineTask::SetModelViewMatrix(matrix)));
    }

    pub fn set_projection_matrix(&mut self, matrix: Mat4f32) {
        self.renderer.worker.push_task(self.id, WorkerTask::PipelineTask(PipelineTask::SetProjectionMatrix(matrix)));
    }

    pub fn record_object(&mut self, object: &ObjectData) {
        let vertex_size = self.pipeline.get_type_table()[object.type_id as usize].vertex_stride;

        let vertex_buffer = self.push_data(object.vertex_data, vertex_size);
        let index_buffer = self.push_data(object.index_data, 4);

        let draw_task = DrawTask {
            vertex_buffer: vertex_buffer.buffer,
            index_buffer: index_buffer.buffer,
            vertex_offset: (vertex_buffer.offset / (vertex_size as usize)) as i32,
            first_index: (index_buffer.offset / 4usize) as u32,
            index_type: vk::IndexType::UINT32,
            index_count: object.index_count,
            type_id: object.type_id,
        };
        self.renderer.worker.push_task(self.id, WorkerTask::PipelineTask(PipelineTask::Draw(draw_task)));
    }

    fn push_data(&mut self, data: &[u8], alignment: u32) -> BufferAllocation {
        let alloc = self.allocate_memory(data.len(), alignment);

        let mut staging = self.renderer.device.get_transfer().request_staging_memory(data.len());
        staging.write(data);
        staging.copy_to_buffer(alloc.buffer, BufferTransferRanges::new_single(
            0,
            alloc.offset as vk::DeviceSize,
            data.len() as vk::DeviceSize
        ));

        alloc
    }

    fn allocate_memory(&mut self, size: usize, alignment: u32) -> BufferAllocation {
        if self.current_buffer.is_none() {
            self.new_sub_allocator(size);
        }

        match self.current_buffer.as_mut().unwrap().allocate(size, alignment) {
            None => {
                self.new_sub_allocator(size);
                self.current_buffer.as_mut().unwrap().allocate(size, alignment).unwrap()
            }
            Some(alloc) => {
                alloc
            }
        }
    }

    fn new_sub_allocator(&mut self, min_size: usize) {
        if let Some(old) = self.current_buffer.take() {
            self.end_sub_allocator(old);
        }

        let (buffer, size, wait_op) = self.renderer.buffer_pool.lock().unwrap().get_buffer(min_size);
        let op = self.renderer.device.get_transfer().prepare_buffer_acquire(buffer, None);
        self.renderer.device.get_transfer().acquire_buffer(op, SemaphoreOps::from_option(wait_op)).unwrap();

        let op = self.renderer.device.get_transfer().prepare_buffer_release(buffer, Some((
            vk::PipelineStageFlags2::VERTEX_INPUT | vk::PipelineStageFlags2::INDEX_INPUT,
            vk::AccessFlags2::VERTEX_ATTRIBUTE_READ | vk::AccessFlags2::INDEX_READ,
            self.renderer.worker.get_render_queue_family()
        )));
        self.renderer.worker.push_task(self.id, WorkerTask::UseDynamicBuffer(buffer, op.get_barrier().cloned()));

        let allocator = BufferSubAllocator::new(buffer, size);

        self.current_buffer = Some(allocator)
    }

    fn end_sub_allocator(&self, allocator: BufferSubAllocator) {
        let transfer = self.renderer.device.get_transfer();

        let buffer = allocator.get_buffer();
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

impl Drop for PassRecorder {
    fn drop(&mut self) {
        if let Some(alloc) = self.current_buffer.take() {
            self.end_sub_allocator(alloc);
        }
        self.renderer.worker.push_task(self.id, WorkerTask::EndPass);
    }
}