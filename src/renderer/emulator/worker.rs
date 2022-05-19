use std::collections::VecDeque;
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Condvar, Mutex};
use ash::vk;
use crate::device::device::VkQueue;

use crate::prelude::*;
use crate::renderer::emulator::pass::{PassId, PassEventListener};
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
    pub fn get_frame_end_semaphore(&self, id: PassId) -> SemaphoreOp {
        SemaphoreOp::new_timeline(self.frame_semaphore, id.get_raw())
    }

    pub fn start_frame(&self, id: PassId, configuration: Arc<RenderConfiguration>, index: usize, signal_semaphore: vk::Semaphore, signal_value: u64) {
        self.push_task(id, Task::StartFrame(configuration, index, signal_semaphore, signal_value));
    }

    pub fn end_frame(&self, id: PassId) {
        self.push_task(id, Task::EndFrame);
    }

    pub fn add_output(&self, id: PassId, config: Arc<OutputConfiguration>, dst_image_index: usize, wait_semaphore: vk::Semaphore, wait_value: Option<u64>) {
        self.push_task(id, Task::AddOutput(config, dst_image_index, wait_semaphore, wait_value.unwrap_or(0)));
    }

    pub fn add_signal_op(&self, id: PassId, semaphore: vk::Semaphore, value: Option<u64>) {
        self.push_task(id, Task::AddSignalOp(semaphore, value.unwrap_or(0)));
    }

    pub fn add_event_listener(&self, id: PassId, listener: Box<dyn PassEventListener + Send + Sync>) {
        self.push_task(id, Task::AddEventListener(listener));
    }

    pub fn use_dynamic_buffer(&self, id: PassId, buffer: Buffer) {
        todo!()
    }

    pub fn set_dynamic_buffer_wait(&self, id: PassId, buffer: BufferId, wait_op: SemaphoreOp) {
        todo!()
    }

    pub fn draw(&self, id: PassId, task: DrawTask) {
        todo!()
    }

    fn push_task(&self, id: PassId, task: Task) {
        self.channel.lock().unwrap().queue.push_back((id, task));
        self.signal.notify_one();
    }

    fn get_next_task(&self) -> (PassId, Task) {
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
    queue: VecDeque<(PassId, Task)>,
}

impl Channel {
    fn new() -> Self {
        Self {
            queue: VecDeque::new()
        }
    }
}

enum Task {
    StartFrame(Arc<RenderConfiguration>, usize, vk::Semaphore, u64),
    EndFrame,
    AddOutput(Arc<OutputConfiguration>, usize, vk::Semaphore, u64),
    AddSignalOp(vk::Semaphore, u64),
    AddEventListener(Box<dyn PassEventListener + Send + Sync>),
    UseDynamicBuffer(Buffer),
    SetDynamicBufferWait(BufferId, SemaphoreOp),
    Draw(DrawTask),
}

impl Debug for Task {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match &self {
            Self::StartFrame(_, _, _, _) => "StartFrame",
            Self::EndFrame => "EndFrame",
            Self::AddOutput(_, _, _, _) => "AddOutput",
            Self::AddSignalOp(_, _) => "AddSignalOp",
            Self::AddEventListener(_) => "AddEventListener",
            _ => "unknown"
        };

        f.write_str(str)
    }
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

    let queue = device.get_device().get_main_queue();

    loop {
        old_frames.retain(|old: &FrameState| {
            if old.is_done(device.get_device()) {
                pool.return_buffer(old.command_buffer);
                false
            } else {
                true
            }
        });

        let (id, task) = share.get_next_task();

        match task {
            Task::StartFrame(config, index, signal_semaphore, signal_value) => {
                let frame = FrameState::new(device.get_device(), config, index, signal_semaphore, signal_value, pool.get_buffer());
                frames.add_frame(id, frame);
            }
            Task::EndFrame => {
                let mut frame = frames.pop_frame(id).unwrap();
                frame.finish_and_submit(device.get_device(), &queue);
                old_frames.push(frame);
            }
            Task::AddOutput(config, index, wait_semaphore, wait_value) =>
                frames.get_frame(id).unwrap().add_output(config, index, wait_semaphore, wait_value),

            Task::AddSignalOp(semaphore, value) =>
                frames.get_frame(id).unwrap().add_signal_op(semaphore, value),

            Task::AddEventListener(listener) =>
                frames.get_frame(id).unwrap().add_event_listener(listener),

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
    frames: Vec<(PassId, FrameState)>,
}

impl Frames {
    fn new() -> Self {
        Self {
            frames: Vec::new(),
        }
    }

    fn add_frame(&mut self, id: PassId, state: FrameState) {
        self.frames.push((id, state))
    }

    fn pop_frame(&mut self, id: PassId) -> Option<FrameState> {
        let mut index = None;
        for (frame_index, (frame_id, _)) in self.frames.iter().enumerate() {
            if id == *frame_id {
                index = Some(frame_index);
            }
        }

        index.map(|index| self.frames.swap_remove(index).1)
    }

    fn get_frame(&mut self, id: PassId) -> Option<&mut FrameState> {
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
    index: usize,
    output_configs: Vec<(Arc<OutputConfiguration>, usize)>,
    wait_semaphores: Vec<vk::Semaphore>,
    wait_values: Vec<u64>,
    signal_semaphores: Vec<vk::Semaphore>,
    signal_values: Vec<u64>,
    event_listeners: Vec<Box<dyn PassEventListener + Send + Sync>>,
    command_buffer: vk::CommandBuffer,

    end_semaphore: vk::Semaphore,
    end_value: u64,
}

impl FrameState {
    pub fn new(device: &DeviceContext, config: Arc<RenderConfiguration>, index: usize, signal_semaphore: vk::Semaphore, signal_value: u64, command_buffer: vk::CommandBuffer) -> Self {
        let info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            device.vk().begin_command_buffer(command_buffer, &info)
        }.unwrap();

        Self {
            config,
            index,
            output_configs: Vec::with_capacity(4),
            wait_semaphores: Vec::new(),
            wait_values: Vec::new(),
            signal_semaphores: vec![signal_semaphore],
            signal_values: vec![signal_value],
            event_listeners: Vec::new(),
            command_buffer,

            end_semaphore: signal_semaphore,
            end_value: signal_value,
        }
    }

    pub fn add_output(&mut self, config: Arc<OutputConfiguration>, index: usize, wait_semaphore: vk::Semaphore, wait_value: u64) {
        self.output_configs.push((config, index));
        self.wait_semaphores.push(wait_semaphore);
        self.wait_values.push(wait_value);
    }

    pub fn add_signal_op(&mut self, semaphore: vk::Semaphore, value: u64) {
        self.signal_semaphores.push(semaphore);
        self.signal_values.push(value);
    }

    pub fn add_event_listener(&mut self, listener: Box<dyn PassEventListener + Send + Sync>) {
        self.event_listeners.push(listener);
    }

    pub fn finish_and_submit(&mut self, device: &DeviceContext, queue: &VkQueue) {
        self.config.record(self.command_buffer, self.index);

        for (output, index) in &self.output_configs {
            output.record(self.command_buffer, self.index, *index);
        }

        unsafe {
            device.vk().end_command_buffer(self.command_buffer)
        }.map_err(|err| {log::error!("Failed to end command buffer recording!"); err}).unwrap();

        let mut timeline_info = vk::TimelineSemaphoreSubmitInfo::builder()
            .wait_semaphore_values(self.wait_values.as_slice())
            .signal_semaphore_values(self.signal_values.as_slice());

        let stages: Box<_> = std::iter::repeat(vk::PipelineStageFlags::ALL_COMMANDS).take(self.wait_values.len()).collect();
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(self.wait_semaphores.as_slice())
            .wait_dst_stage_mask(stages.as_ref())
            .command_buffers(std::slice::from_ref(&self.command_buffer))
            .signal_semaphores(self.signal_semaphores.as_slice())
            .push_next(&mut timeline_info);

        unsafe {
            queue.submit(std::slice::from_ref(&submit_info), None)
        }.map_err(|err| {log::error!("Failed to submit command buffer!"); err}).unwrap();

        for listener in &self.event_listeners {
            listener.on_post_submit();
        }
    }

    pub fn is_done(&self, device: &DeviceContext) -> bool {
        let value = unsafe {
            device.vk().get_semaphore_counter_value(self.end_semaphore)
        }.unwrap();

        value >= self.end_value
    }
}

impl Drop for FrameState {
    fn drop(&mut self) {
        for listener in &self.event_listeners {
            listener.on_execution_completed();
        }
    }
}