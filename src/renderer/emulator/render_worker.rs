use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};
use ash::vk;

use crate::prelude::*;
use crate::renderer::emulator::frame::FrameId;
use crate::renderer::emulator::pipeline::PipelineId;
use crate::vk::objects::buffer::{Buffer, BufferId};
use crate::vk::objects::semaphore::SemaphoreOp;

pub struct Share {
    frame_semaphore: vk::Semaphore,
    channel: Mutex<Channel>,
    signal: Condvar,
}

impl Share {
    pub fn new() -> Self {
        todo!()
    }

    pub fn get_render_queue_family(&self) -> u32 {
        todo!()
    }

    /// Returns a timeline semaphore op that can be used to determine when a frame has finished
    /// rendering.
    ///
    /// It is guaranteed that all resources associated with the frame may be freed safely after
    /// the semaphore is triggered.
    pub fn get_frame_end_semaphore(&self, id: FrameId) -> SemaphoreOp {
        SemaphoreOp::new_timeline(self.frame_semaphore, id.get_raw())
    }

    pub fn start_frame(&self, id: FrameId) {
        todo!()
    }

    pub fn submit_frame(&self, id: FrameId) {
        todo!()
    }

    pub fn use_dynamic_buffer(&self, id: FrameId, buffer: Buffer) {
        todo!()
    }

    pub fn set_dynamic_buffer_wait(&self, id: FrameId, buffer: BufferId, wait_op: SemaphoreOp) {
        todo!()
    }

    pub fn draw(&self, id: FrameId, task: DrawTask) {
        todo!()
    }

    fn push_task(&self, task: Task) {
        self.channel.lock().unwrap().queue.push_back(task);
        self.signal.notify_one();
    }
}

struct Channel {
    queue: VecDeque<Task>,
}

enum Task {
    StartFrame(FrameId),
    SubmitFrame(FrameId),
    UseDynamicBuffer(FrameId, Buffer),
    SetDynamicBufferWait(FrameId, BufferId, SemaphoreOp),
    Draw(FrameId, DrawTask),
}

pub struct DrawTask {
    pub vertex_buffer: BufferId,
    pub index_buffer: BufferId,
    pub first_vertex: u32,
    pub first_index: u32,
    pub vertex_count: u32,
    pub pipeline: PipelineId,
}

struct RenderWorker {

}