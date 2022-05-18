use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use ash::vk;
use crate::device::device::VkQueue;

use crate::prelude::*;
use crate::renderer::emulator::frame::FrameId;
use crate::renderer::emulator::{EmulatorRenderer, OutputConfiguration, RenderConfiguration};
use crate::vk::DeviceEnvironment;
use crate::vk::objects::buffer::{Buffer, BufferId};
use crate::vk::objects::semaphore::SemaphoreOp;

pub struct Share {
    frame_semaphore: vk::Semaphore,
    channel: Mutex<Channel>,
    signal: Condvar,
}

impl Share {
    pub fn new(device: DeviceEnvironment) -> Self {
        let mut timeline = vk::SemaphoreTypeCreateInfo::builder()
            .semaphore_type(vk::SemaphoreType::TIMELINE)
            .initial_value(0);

        let info = vk::SemaphoreCreateInfo::builder()
            .push_next(&mut timeline);

        let frame_semaphore = unsafe {
            device.vk().create_semaphore(&info, None)
        }.unwrap();

        Self {
            frame_semaphore,
            channel: Mutex::new(Channel::new()),
            signal: Condvar::new(),
        }
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

    pub fn start_frame(&self, id: FrameId, configuration: Arc<RenderConfiguration>) {
        self.push_task(id, Task::StartFrame(configuration));
    }

    pub fn end_frame(&self, id: FrameId) {
        self.push_task(id, Task::EndFrame);
    }

    pub fn add_output(&self, id: FrameId, config: Arc<OutputConfiguration>, dst_image_index: usize) {
        self.push_task(id, Task::AddOutput(config, dst_image_index));
    }

    pub fn add_signal_op(&self, id: FrameId, semaphore: vk::Semaphore, value: Option<u64>) {
        self.push_task(id, Task::AddSignalOp(semaphore, value.unwrap_or(0)));
    }

    pub fn add_post_fn(&self, id: FrameId, func: Box<dyn FnOnce() + Send + Sync>) {
        self.push_task(id, Task::AddPostFn(func));
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

    fn push_task(&self, id: FrameId, task: Task) {
        self.channel.lock().unwrap().queue.push_back((id, task));
        self.signal.notify_one();
    }

    fn get_next_task(&self) -> (FrameId, Task) {
        let mut guard = self.channel.lock().unwrap();
        loop {
            if let Some(task) = guard.queue.pop_front() {
                return task;
            }
            guard = self.signal.wait(guard).unwrap();
        }
    }
}

struct Channel {
    queue: VecDeque<(FrameId, Task)>,
}

impl Channel {
    fn new() -> Self {
        Self {
            queue: VecDeque::new()
        }
    }
}

enum Task {
    StartFrame(Arc<RenderConfiguration>),
    EndFrame,
    AddOutput(Arc<OutputConfiguration>, usize),
    AddSignalOp(vk::Semaphore, u64),
    AddPostFn(Box<dyn FnOnce() + Send + Sync>),
    UseDynamicBuffer(Buffer),
    SetDynamicBufferWait(BufferId, SemaphoreOp),
    Draw(DrawTask),
}

pub struct DrawTask {
    pub vertex_buffer: BufferId,
    pub index_buffer: BufferId,
    pub first_vertex: u32,
    pub first_index: u32,
    pub vertex_count: u32,
}

pub(super) fn run_worker(device: DeviceEnvironment, share: Arc<Share>) {
    let mut pool = CommandPool::new(device.get_device().clone());
    let mut frames = Frames::new();
    let mut old_frames = Vec::new();

    loop {
        let (id, task) = share.get_next_task();
        match task {
            Task::StartFrame(config) => {
                let frame = FrameState::new(config, pool.get_buffer());
                frames.add_frame(id, frame);
            }
            Task::EndFrame => {
                let mut frame = frames.pop_frame(id).unwrap();
                frame.finish_and_submit();
                old_frames.push(frame);
            }
            Task::AddOutput(config, index) =>
                frames.get_frame(id).unwrap().add_output(config, index),

            Task::AddSignalOp(semaphore, value) =>
                frames.get_frame(id).unwrap().add_signal_op(semaphore, value),

            Task::AddPostFn(func) => {}

            Task::UseDynamicBuffer(_) => {}
            Task::SetDynamicBufferWait(_, _) => {}
            Task::Draw(_) => {}
        }
    }
}

struct CommandPool {
    device: Arc<DeviceContext>,
    queue: VkQueue,
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
}

impl CommandPool {
    fn new(device: Arc<DeviceContext>) -> Self {
        let queue = device.get_main_queue();

        let info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER | vk::CommandPoolCreateFlags::TRANSIENT)
            .queue_family_index(queue.get_queue_family_index());

        let command_pool = unsafe {
            device.vk().create_command_pool(&info, None)
        }.unwrap();

        Self {
            device,
            queue,
            command_pool,
            command_buffers: Vec::new(),
        }
    }

    fn get_buffer(&mut self) -> vk::CommandBuffer {
        if self.command_buffers.is_empty() {
            let info = vk::CommandBufferAllocateInfo::builder()
                .command_pool(self.command_pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(1);

            let buffer = *unsafe {
                self.device.vk().allocate_command_buffers(&info).unwrap().get(0).unwrap()
            };

            self.command_buffers.push(buffer);
        }

        self.command_buffers.pop().unwrap()
    }

    fn return_buffer(&mut self, buffer: vk::CommandBuffer) {
        self.command_buffers.push(buffer)
    }
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().destroy_command_pool(self.command_pool, None)
        };
    }
}

struct Frames {
    frames: Vec<(FrameId, FrameState)>,
}

impl Frames {
    fn new() -> Self {
        Self {
            frames: Vec::new(),
        }
    }

    fn add_frame(&mut self, id: FrameId, state: FrameState) {
        self.frames.push((id, state))
    }

    fn pop_frame(&mut self, id: FrameId) -> Option<FrameState> {
        let mut index = None;
        for (frame_index, (frame_id, _)) in self.frames.iter().enumerate() {
            if id == *frame_id {
                index = Some(frame_index);
            }
        }

        index.map(|index| self.frames.swap_remove(index).1)
    }

    fn get_frame(&mut self, id: FrameId) -> Option<&mut FrameState> {
        for (frame_id, frame) in &mut self.frames {
            if id == *frame_id {
                return Some(frame);
            }
        }
        None
    }
}

struct FrameState {
    config: Arc<RenderConfiguration>,
    output_configs: Vec<(Arc<OutputConfiguration>, usize)>,
    signal_ops: Vec<(vk::Semaphore, u64)>,
    post_ops: Vec<Box<dyn FnOnce() + Send + Sync>>,
    command_buffer: vk::CommandBuffer,
}

impl FrameState {
    pub fn new(config: Arc<RenderConfiguration>, command_buffer: vk::CommandBuffer) -> Self {

        Self {
            config,
            output_configs: Vec::with_capacity(4),
            signal_ops: Vec::new(),
            post_ops: Vec::new(),
            command_buffer,
        }
    }

    pub fn add_output(&mut self, config: Arc<OutputConfiguration>, index: usize) {
        self.output_configs.push((config, index));
    }

    pub fn add_signal_op(&mut self, semaphore: vk::Semaphore, value: u64) {
        self.signal_ops.push((semaphore, value));
    }

    pub fn add_post_fn(&mut self, func: Box<dyn FnOnce() + Send + Sync>) {
        self.post_ops.push(func);
    }

    pub fn finish_and_submit(&mut self) {
    }
}