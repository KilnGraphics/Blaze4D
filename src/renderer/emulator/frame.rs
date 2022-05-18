use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use ash::vk;
use crate::renderer::emulator::buffer::{BufferAllocation, BufferSubAllocator};
use crate::renderer::emulator::{EmulatorRenderer, RenderConfiguration};
use crate::renderer::emulator::pipeline::PipelineId;
use crate::renderer::emulator::render_worker::DrawTask;
use crate::device::transfer::{BufferAvailabilityOp, BufferTransferRanges};
use crate::vk::objects::semaphore::SemaphoreOps;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct FrameId(u64);

impl FrameId {
    pub fn from_raw(id: u64) -> Self {
        Self(id)
    }

    pub fn get_raw(&self) -> u64 {
        self.0
    }
}

struct FrameShare {
    id: FrameId,
    renderer: Arc<EmulatorRenderer>,
    buffers: BufferSubAllocator,
}

impl FrameShare {
    pub fn record_object(&mut self, object: &ObjectData) {
        let vertex_size = self.get_pipeline_vertex_size(object.pipeline);
        let vertex_buffer = self.push_data(object.vertex_data, vertex_size);
        let index_buffer = self.push_data(object.index_data, 4);

        let draw_task = DrawTask {
            vertex_buffer: vertex_buffer.buffer.get_id(),
            index_buffer: index_buffer.buffer.get_id(),
            first_vertex: (vertex_buffer.offset / (vertex_size as usize)) as u32,
            first_index: (index_buffer.offset / 4usize) as u32,
            vertex_count: object.draw_count,
            pipeline: object.pipeline
        };
        self.renderer.worker.draw(self.id, draw_task);
    }

    fn push_data(&mut self, data: &[u8], alignment: u32) -> BufferAllocation {
        let alloc = {
            match self.buffers.allocate(data.len(), alignment) {
                None => {
                    self.replace_sub_allocator(data.len());
                    self.buffers.allocate(data.len(), alignment).expect("Failed to allocate after replacement")
                },
                Some(alloc) => alloc,
            }
        };

        let mut staging = self.renderer.device.get_transfer().request_staging_memory(data.len());
        staging.write(data);
        staging.copy_to_buffer(alloc.buffer, BufferTransferRanges::new_single(
            0,
            alloc.offset as vk::DeviceSize,
            data.len() as vk::DeviceSize
        ));

        alloc
    }

    fn get_pipeline_vertex_size(&self, pipeline: PipelineId) -> u32 {
        todo!()
    }

    fn acquire_sub_allocator(&self, min_size: usize) -> BufferSubAllocator {
        let (buffer, size, wait_op) = self.renderer.buffer_pool.lock().unwrap().get_buffer(min_size);
        self.renderer.device.get_transfer().make_buffer_available(BufferAvailabilityOp::new(
            buffer, self.renderer.worker.get_render_queue_family(), SemaphoreOps::from_option(wait_op)
        ));
        self.renderer.worker.use_dynamic_buffer(self.id, buffer);

        BufferSubAllocator::new(buffer, size)
    }

    fn finish_sub_allocator(&self, allocator: BufferSubAllocator) {
        let buffer = allocator.get_buffer();
        let transfer_id = self.renderer.device.get_transfer().release_buffer(BufferAvailabilityOp::new(
            buffer,
            self.renderer.worker.get_render_queue_family(),
            SemaphoreOps::None
        ));

        let transfer_wait_op = self.renderer.device.get_transfer().get_wait_op(transfer_id);
        self.renderer.worker.set_dynamic_buffer_wait(self.id, buffer.get_id(), transfer_wait_op);
    }

    fn replace_sub_allocator(&mut self, min_size: usize) {
        let new = self.acquire_sub_allocator(min_size);
        let old = std::mem::replace(&mut self.buffers, new);
        self.finish_sub_allocator(old);
    }
}

struct ObjectData<'a> {
    vertex_data: &'a [u8],
    index_data: &'a [u8],
    pipeline: PipelineId,
    draw_count: u32,
}

pub struct Frame {
    id: FrameId,
    renderer: Arc<EmulatorRenderer>,
    configuration: Arc<RenderConfiguration>,
}

impl Frame {
    pub(super) fn new(id: FrameId, renderer: Arc<EmulatorRenderer>, configuration: Arc<RenderConfiguration>) -> Self {
        Self {
            id,
            renderer,
            configuration,
        }
    }
}