use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use ash::vk;
use json::object;
use crate::renderer::emulator::buffer::{BufferAllocation, BufferSubAllocator};
use crate::renderer::emulator::{EmulatorRenderer, OutputConfiguration, RenderConfiguration};
use crate::renderer::emulator::worker::DrawTask;
use crate::device::transfer::{BufferAvailabilityOp, BufferTransferRanges};
use crate::objects::sync::SemaphoreOps;
use crate::prelude::Vec2u32;

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
    pub draw_count: u32,
}

pub struct Pass {
    id: PassId,
    renderer: Arc<EmulatorRenderer>,
    configuration: Arc<RenderConfiguration>,
    current_buffer: Option<BufferSubAllocator>,
}

impl Pass {
    pub(super) fn new(id: PassId, renderer: Arc<EmulatorRenderer>, configuration: Arc<RenderConfiguration>) -> Self {
        let index = configuration.get_next_index();
        let (signal_semaphore, signal_value) = configuration.prepare_index(index);

        renderer.worker.start_frame(id, configuration.clone(), index, signal_semaphore, signal_value);

        Self {
            id,
            renderer,
            configuration,
            current_buffer: None,
        }
    }

    pub fn add_output(&self, output: Arc<OutputConfiguration>, dst_image_index: usize, wait_semaphore: vk::Semaphore, wait_value: Option<u64>) {
        self.renderer.worker.add_output(self.id, output, dst_image_index, wait_semaphore, wait_value);
    }

    pub fn add_signal_op(&self, semaphore: vk::Semaphore, value: Option<u64>) {
        self.renderer.worker.add_signal_op(self.id, semaphore, value);
    }

    pub fn add_event_listener(&self, listener: Box<dyn PassEventListener + Send + Sync>) {
        self.renderer.worker.add_event_listener(self.id, listener);
    }

    pub fn record_object(&mut self, object: &ObjectData) {
        let vertex_size = self.configuration.get_tmp_vertex_size();
        let vertex_buffer = self.push_data(object.vertex_data, vertex_size as u32);
        let index_buffer = self.push_data(object.index_data, 4);

        let draw_task = DrawTask {
            vertex_buffer: vertex_buffer.buffer.get_id(),
            index_buffer: index_buffer.buffer.get_id(),
            first_vertex: (vertex_buffer.offset / (vertex_size as usize)) as u32,
            first_index: (index_buffer.offset / 4usize) as u32,
            vertex_count: object.draw_count,
        };
        self.renderer.worker.draw(self.id, draw_task);
    }

    pub fn submit(self) {
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
        self.renderer.worker.use_dynamic_buffer(self.id, buffer);

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
        let release_id = transfer.release_buffer(op.clone()).unwrap();

        let transfer_wait_op = transfer.generate_wait_semaphore(release_id);
        self.renderer.worker.set_dynamic_buffer_wait(self.id, buffer.get_id(), transfer_wait_op);
    }
}

impl Drop for Pass {
    fn drop(&mut self) {
        if let Some(alloc) = self.current_buffer.take() {
            self.end_sub_allocator(alloc);
        }
        // self.renderer.device.get_transfer().flush();
        self.renderer.worker.end_frame(self.id);
    }
}

pub trait PassEventListener {
    /// Called after all command buffers have been submitted for execution
    fn on_post_submit(&self);

    /// Called after all submitted commands have finished execution. It is guaranteed that any
    /// external object which has be passed to the pass can now be used or destroyed.
    fn on_execution_completed(&self);
}