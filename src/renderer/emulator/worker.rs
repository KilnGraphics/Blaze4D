use std::collections::VecDeque;
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Condvar, Mutex};
use ash::vk;
use crate::device::device::VkQueue;
use crate::objects::sync::SemaphoreOp;

use crate::prelude::*;
use crate::renderer::emulator::pass::{PassId, PassEventListener};
use crate::renderer::emulator::{EmulatorRenderer, OutputConfiguration, RenderConfiguration};
use crate::renderer::emulator::buffer::BufferPool;
use crate::vk::DeviceEnvironment;
use crate::vk::objects::buffer::{Buffer, BufferId};

pub struct Share {
    pool: Arc<Mutex<BufferPool>>,
    frame_semaphore: vk::Semaphore,
    channel: Mutex<Channel>,
    signal: Condvar,
    family: u32,
}

impl Share {
    pub fn new(device: DeviceEnvironment, pool: Arc<Mutex<BufferPool>>) -> Self {
        let mut timeline = vk::SemaphoreTypeCreateInfo::builder()
            .semaphore_type(vk::SemaphoreType::TIMELINE)
            .initial_value(0);

        let info = vk::SemaphoreCreateInfo::builder()
            .push_next(&mut timeline);

        let frame_semaphore = unsafe {
            device.vk().create_semaphore(&info, None)
        }.unwrap();

        Self {
            pool,
            frame_semaphore,
            channel: Mutex::new(Channel::new()),
            signal: Condvar::new(),
            family: device.get_device().get_main_queue().get_queue_family_index(),
        }
    }

    pub fn get_render_queue_family(&self) -> u32 {
        self.family
    }

    /// Returns a timeline semaphore op that can be used to determine when a frame has finished
    /// rendering.
    ///
    /// It is guaranteed that all resources associated with the frame may be freed safely after
    /// the semaphore is triggered.
    pub fn get_frame_end_semaphore(&self, id: PassId) -> SemaphoreOp {
        todo!()
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
        self.push_task(id, Task::UseDynamicBuffer(buffer));
    }

    pub fn set_dynamic_buffer_wait(&self, id: PassId, buffer: BufferId, wait_op: SemaphoreOp) {
        self.push_task(id, Task::SetDynamicBufferWait(buffer, wait_op));
    }

    pub fn draw(&self, id: PassId, task: DrawTask) {
        self.push_task(id, Task::Draw(task));
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
    SetWorldNdcMat(Mat4f32),
    SetModelWorldMat(Mat4f32),
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
                frame.finish_and_submit(&device, &queue, &share.pool);
                old_frames.push(frame);
            }
            Task::AddOutput(config, index, wait_semaphore, wait_value) =>
                frames.get_frame(id).unwrap().add_output(config, index, wait_semaphore, wait_value),

            Task::AddSignalOp(semaphore, value) =>
                frames.get_frame(id).unwrap().add_signal_op(semaphore, value),

            Task::AddEventListener(listener) =>
                frames.get_frame(id).unwrap().add_event_listener(listener),

            Task::SetWorldNdcMat(mat) =>
                frames.get_frame(id).unwrap().set_world_ndc_mat(mat),

            Task::SetModelWorldMat(mat) =>
                frames.get_frame(id).unwrap().set_model_world_mat(mat),

            Task::UseDynamicBuffer(buffer) =>
                frames.get_frame(id).unwrap().use_dynamic_buffer(&device, buffer),

            Task::SetDynamicBufferWait(buffer, wait) =>
                frames.get_frame(id).unwrap().set_dynamic_buffer_wait(buffer, wait),

            Task::Draw(task) =>
                frames.get_frame(id).unwrap().draw(&device, task),
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

    available_buffers: Vec<Buffer>,

    current_vertex_buffer: Option<BufferId>,
    current_index_buffer: Option<BufferId>,

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

        config.begin_render_pass(command_buffer, index);

        let mat = Mat4f32::identity();
        config.set_world_ndc_mat(command_buffer, mat);
        config.set_model_world_mat(command_buffer, mat);

        config.test_draw(command_buffer);

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

            available_buffers: Vec::new(),

            current_vertex_buffer: None,
            current_index_buffer: None,

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

    pub fn set_world_ndc_mat(&self, mat: Mat4f32) {
        self.config.set_world_ndc_mat(self.command_buffer, mat);
    }

    pub fn set_model_world_mat(&self, mat: Mat4f32) {
        self.config.set_model_world_mat(self.command_buffer, mat);
    }

    pub fn use_dynamic_buffer(&mut self, device: &DeviceEnvironment, buffer: Buffer) {
        self.available_buffers.push(buffer);

        if device.get_transfer().get_queue_family() != device.get_device().get_main_queue().get_queue_family_index() {
            let buffer_barrier = vk::BufferMemoryBarrier::builder()
                .buffer(buffer.get_handle())
                .offset(0)
                .size(vk::WHOLE_SIZE)
                .dst_access_mask(vk::AccessFlags::MEMORY_READ)
                .src_queue_family_index(device.get_transfer().get_queue_family())
                .dst_queue_family_index(device.get_device().get_main_queue().get_queue_family_index());

            unsafe {
                device.vk().cmd_pipeline_barrier(
                    self.command_buffer,
                    vk::PipelineStageFlags::NONE,
                    vk::PipelineStageFlags::ALL_GRAPHICS,
                    vk::DependencyFlags::empty(),
                    &[],
                    std::slice::from_ref(&buffer_barrier),
                    &[]
                )
            };
        }
    }

    pub fn set_dynamic_buffer_wait(&mut self, _: BufferId, wait_op: SemaphoreOp) {
        todo!()
    }

    pub fn draw(&mut self, device: &DeviceEnvironment, task: DrawTask) {
        if self.current_vertex_buffer != Some(task.vertex_buffer) {
            unsafe {
                device.vk().cmd_bind_vertex_buffers(self.command_buffer, 0, std::slice::from_ref(&self.find_buffer(task.vertex_buffer)), &[0])
            };
            self.current_vertex_buffer = Some(task.vertex_buffer);
        }
        if self.current_index_buffer != Some(task.index_buffer) {
            unsafe {
                device.vk().cmd_bind_index_buffer(self.command_buffer, self.find_buffer(task.index_buffer), 0, vk::IndexType::UINT32)
            };
            self.current_index_buffer = Some(task.index_buffer);
        }

        unsafe {
            device.vk().cmd_draw_indexed(self.command_buffer, task.vertex_count, 1, task.first_index, task.first_vertex as i32, 0)
        };
    }

    pub fn finish_and_submit(&mut self, device: &DeviceEnvironment, queue: &VkQueue, buffer_pool: &Mutex<BufferPool>) {
        unsafe {
            device.vk().cmd_end_render_pass(self.command_buffer);
        }

        for (output, index) in &self.output_configs {
            output.record(self.command_buffer, self.index, *index);
        }

        if device.get_transfer().get_queue_family() != queue.get_queue_family_index() {
            for buffer in &self.available_buffers {
                let barrier = vk::BufferMemoryBarrier::builder()
                    .buffer(buffer.get_handle())
                    .offset(0)
                    .size(vk::WHOLE_SIZE)
                    .src_access_mask(vk::AccessFlags::MEMORY_READ)
                    .src_queue_family_index(queue.get_queue_family_index())
                    .dst_queue_family_index(device.get_transfer().get_queue_family());

                unsafe {
                    device.vk().cmd_pipeline_barrier(
                        self.command_buffer,
                        vk::PipelineStageFlags::ALL_GRAPHICS,
                        vk::PipelineStageFlags::NONE,
                        vk::DependencyFlags::empty(),
                        &[],
                        std::slice::from_ref(&barrier),
                        &[]
                    )
                };
            }
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

        let mut pool_guard = buffer_pool.lock().unwrap();
        for buffer in &self.available_buffers {
            todo!()
        }
    }

    pub fn is_done(&self, device: &DeviceContext) -> bool {
        let value = unsafe {
            device.vk().get_semaphore_counter_value(self.end_semaphore)
        }.unwrap();

        value >= self.end_value
    }

    fn find_buffer(&self, id: BufferId) -> vk::Buffer {
        for buffer in &self.available_buffers {
            if buffer.get_id() == id {
                return buffer.get_handle();
            }
        }
        log::error!("Failed to find buffer {:?} in {:?}", id, self.available_buffers);
        panic!();
    }
}

impl Drop for FrameState {
    fn drop(&mut self) {
        for listener in &self.event_listeners {
            listener.on_execution_completed();
        }
    }
}