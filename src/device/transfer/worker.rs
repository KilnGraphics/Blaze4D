use std::time::{Duration, Instant};

use crate::device::transfer::resource_state::{BufferStateTracker, ImageStateTracker};
use crate::vk::objects::semaphore::SemaphoreOp;

use super::*;

#[derive(Debug)]
pub enum TaskInfo {
    Flush,
    BufferAcquire(BufferAvailabilityOp),
    BufferRelease(BufferAvailabilityOp),
    ImageAcquire(ImageAvailabilityOp),
    ImageRelease(ImageAvailabilityOp),
    BufferTransfer(BufferTransfer),
    BufferToImageTransfer(BufferToImageTransfer),
    ImageToBufferTransfer(ImageToBufferTransfer),
    AcquireStagingMemory(Buffer),
    FreeStagingMemory(Buffer, Allocation),
}

#[derive(Debug)]
pub struct Task {
    pub id: u64,
    pub info: TaskInfo,
}

pub struct Channel {
    pub task_queue: VecDeque<Task>,
    pub current_task_id: u64,
    pub auto_submit_interval: Duration,
}

pub(super) fn run_worker(share: Arc<Transfer>, queue: VkQueue) {
    let mut worker = TransferWorker::new(share, queue);
    worker.run();
}

struct TransferWorker {
    share: Arc<Transfer>,
    device: Arc<DeviceContext>,
    allocator: Arc<Allocator>,
    queue: VkQueue,

    buffer_states: BufferStateTracker,
    image_states: ImageStateTracker,

    command_pool: vk::CommandPool,
    available_buffers: VecDeque<vk::CommandBuffer>,
    submitted_buffers: VecDeque<(u64, vk::CommandBuffer, Vec<(Buffer, Allocation)>)>,

    current_buffer: Option<vk::CommandBuffer>,
    has_commands: bool,
    current_task_id: u64,
    last_task_id: u64,
    last_start: Option<Instant>,

    buffer_barriers: Vec<vk::BufferMemoryBarrier2>,
    image_barriers: Vec<vk::ImageMemoryBarrier2>,

    free_ops: Vec<(Buffer, Allocation)>,
    wait_ops: Vec<vk::SemaphoreSubmitInfo>,
    signal_ops: Vec<vk::SemaphoreSubmitInfo>,

    vk: ash::Device,
}

impl TransferWorker {
    fn new(share: Arc<Transfer>, queue: VkQueue) -> Self {
        let device = share.device.clone();
        let allocator = share.allocator.clone();

        let info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::TRANSIENT | vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue.get_queue_family_index());

        let command_pool = unsafe {
            device.vk().create_command_pool(&info, None)
        }.unwrap();

        let info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(8);

        let command_buffers = unsafe {
            device.vk().allocate_command_buffers(&info)
        }.unwrap();

        let vk = device.vk().clone();

        Self {
            share,
            device,
            allocator,
            queue,

            buffer_states: BufferStateTracker::new(),
            image_states: ImageStateTracker::new(),

            command_pool,
            available_buffers: command_buffers.into(),
            submitted_buffers: VecDeque::with_capacity(8),

            current_buffer: None,
            has_commands: false,
            current_task_id: 0,
            last_task_id: 0,
            last_start: None,

            buffer_barriers: Vec::with_capacity(8),
            image_barriers: Vec::with_capacity(8),

            free_ops: Vec::with_capacity(32),
            wait_ops: Vec::with_capacity(16),
            signal_ops: Vec::with_capacity(16),

            vk,
        }
    }

    pub fn run(&mut self) {
        log::info!("Transfer worker started main loop");
        loop {
            let task = self.acquire_task_loop();
            let cmd = self.acquire_command_buffer();

            // If we had to block then we very likely have a long backlog of submissions.
            // Immediately submitting by starting the timer before waiting for a command buffer
            // would be very counter productive then.
            if self.last_start.is_none() {
                self.last_start = Some(Instant::now())
            }

            log::error!("Transfer task {:?}", &task);
            self.current_task_id = task.id;
            match task.info {
                TaskInfo::Flush => self.submit(),
                TaskInfo::BufferAcquire(op) => self.process_buffer_acquire(&op),
                TaskInfo::BufferRelease(op) => self.process_buffer_release(&op),
                TaskInfo::ImageAcquire(op) => self.process_image_acquire(&op, cmd),
                TaskInfo::ImageRelease(op) => self.process_image_release(&op),
                TaskInfo::BufferTransfer(transfer) => self.process_buffer_transfer(&transfer, cmd),
                TaskInfo::BufferToImageTransfer(transfer) => self.process_buffer_to_image_transfer(&transfer, cmd),
                TaskInfo::ImageToBufferTransfer(transfer) => todo!(),
                TaskInfo::AcquireStagingMemory(buffer) => self.buffer_states.register(buffer).unwrap(),
                TaskInfo::FreeStagingMemory(buffer, allocation) => {
                    self.buffer_states.release(buffer.get_id()).unwrap();
                    self.free_ops.push((buffer, allocation))
                }
            }
            log::error!("Task completed");
        }
    }

    fn acquire_task_loop(&mut self) -> Task {
        loop {
            // Regularly check if any previous submissions have completed
            self.check_submitted();

            let mut guard = self.share.channel.lock().unwrap();
            if let Some(task) = guard.task_queue.pop_front() {
                return task;
            }

            // Wait until next auto submit
            let timeout = if let Some(start) = self.last_start {
                let elapsed = Instant::now().duration_since(start);
                elapsed.saturating_sub(guard.auto_submit_interval)
            } else {
                guard.auto_submit_interval
            };
            let (mut guard, timeout) = self.share.condvar.wait_timeout(guard, timeout).unwrap();

            if timeout.timed_out() {
                // Make sure we dont block the lock while submitting
                drop(guard);
                self.submit();
                continue;
            }

            if let Some(task) = guard.task_queue.pop_front() {
                return task;
            }
        }
    }

    fn check_submitted(&mut self) {
        loop {
            if self.submitted_buffers.is_empty() {
                return;
            }

            let id = self.submitted_buffers.front().unwrap().0;
            let current_count = unsafe {
                self.vk.get_semaphore_counter_value(self.share.semaphore)
            }.unwrap();

            if id <= current_count {
                let (_, buffer, allocations) = self.submitted_buffers.pop_front().unwrap();
                self.process_completed_submitted(buffer, allocations);
            } else {
                return;
            }
        }
    }

    fn process_completed_submitted(&mut self, buffer: vk::CommandBuffer, allocations: Vec<(Buffer, Allocation)>) {
        unsafe {
            self.vk.reset_command_buffer(buffer, vk::CommandBufferResetFlags::empty())
        }.expect("Failed to reset command buffer");
        self.available_buffers.push_back(buffer);

        self.process_free_ops(allocations);
    }

    fn process_free_ops(&mut self, allocations: Vec<(Buffer, Allocation)>) {
        for (buffer, allocation) in allocations {
            unsafe {
                self.vk.destroy_buffer(buffer.get_handle(), None);
            }
            self.allocator.free(allocation);
        }
    }

    fn acquire_command_buffer(&mut self) -> vk::CommandBuffer {
        if let Some(cmd) = self.current_buffer {
            cmd
        } else {
            loop {
                if let Some(cmd) = self.available_buffers.pop_front() {
                    let begin_info = vk::CommandBufferBeginInfo::builder();
                    unsafe {
                        self.vk.begin_command_buffer(cmd, &begin_info)
                    }.expect("Failed to start command buffer recording");

                    self.current_buffer = Some(cmd);
                    return cmd;
                }

                // TODO we should block here
                self.check_submitted();
            }
        }
    }

    fn process_buffer_acquire(&mut self, op: &BufferAvailabilityOp) {
        self.buffer_states.register(op.buffer).expect("Buffer was already available");

        if op.queue != self.queue.get_queue_family_index() {
            self.buffer_barriers.push(vk::BufferMemoryBarrier2::builder()
                .dst_stage_mask(vk::PipelineStageFlags2::TRANSFER)
                .dst_access_mask(vk::AccessFlags2::TRANSFER_READ | vk::AccessFlags2::TRANSFER_WRITE)
                .src_queue_family_index(op.queue)
                .dst_queue_family_index(self.queue.get_queue_family_index())
                .buffer(op.buffer.get_handle())
                .offset(0)
                .size(vk::WHOLE_SIZE)
                .build()
            );

            self.has_commands = true;
        }

        self.record_wait_ops(op.semaphore_ops.as_slice());
    }

    fn process_buffer_release(&mut self, op: &BufferAvailabilityOp) {
        let (handle, access_mask) = self.buffer_states.release(op.buffer.get_id()).expect("Buffer was not available");

        if op.queue != self.queue.get_queue_family_index() {
            let access_mask = if access_mask.is_empty() { vk::AccessFlags2::TRANSFER_WRITE } else { access_mask };
            self.buffer_barriers.push(vk::BufferMemoryBarrier2::builder()
                .src_stage_mask(vk::PipelineStageFlags2::TRANSFER)
                .src_access_mask(access_mask)
                .src_queue_family_index(self.queue.get_queue_family_index())
                .dst_queue_family_index(op.queue)
                .buffer(handle)
                .offset(0)
                .size(vk::WHOLE_SIZE)
                .build()
            );

            self.has_commands = true;
        }

        self.record_signal_ops(op.semaphore_ops.as_slice());
    }

    fn process_image_acquire(&mut self, op: &ImageAvailabilityOp, cmd: vk::CommandBuffer) {
        self.image_states.register(op.image, op.aspect_mask, op.layout).expect("Image was already available");

        if op.queue != self.queue.get_queue_family_index() {
            self.image_barriers.push(vk::ImageMemoryBarrier2::builder()
                .dst_stage_mask(vk::PipelineStageFlags2::TRANSFER)
                .dst_access_mask(vk::AccessFlags2::TRANSFER_READ | vk::AccessFlags2::TRANSFER_WRITE)
                .src_queue_family_index(op.queue)
                .dst_queue_family_index(self.queue.get_queue_family_index())
                .image(op.image.get_handle())
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: Default::default(),
                    base_mip_level: 0,
                    level_count: 0,
                    base_array_layer: 0,
                    layer_count: 0
                })
                .build()
            );

            self.has_commands = true;
        }

        self.record_wait_ops(op.semaphore_ops.as_slice());

        // Need to flush in case we need to do a layout transition later
        self.flush_barriers(cmd);
    }

    fn process_image_release(&mut self, op: &ImageAvailabilityOp) {
        let (handle, aspect_mask, access_mask, layout) = self.image_states.release(op.image.get_id()).expect("Image was not available");

        if op.queue != self.queue.get_queue_family_index() || op.layout != layout {
            self.image_barriers.push(vk::ImageMemoryBarrier2::builder()
                .src_stage_mask(vk::PipelineStageFlags2::TRANSFER)
                .src_access_mask(access_mask)
                .dst_stage_mask(vk::PipelineStageFlags2::TOP_OF_PIPE)
                .dst_access_mask(vk::AccessFlags2::MEMORY_READ | vk::AccessFlags2::MEMORY_WRITE)
                .old_layout(layout)
                .new_layout(op.layout)
                .image(handle)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask,
                    base_mip_level: 0,
                    level_count: vk::REMAINING_MIP_LEVELS,
                    base_array_layer: 0,
                    layer_count: vk::REMAINING_ARRAY_LAYERS
                })
                .build()
            );
        }

        self.has_commands = true;
    }

    fn process_buffer_transfer(&mut self, op: &BufferTransfer, cmd: vk::CommandBuffer) {
        let src_handle;
        let dst_handle;
        if op.src_buffer == op.dst_buffer {
            src_handle = self.buffer_states.update_state(op.src_buffer, true, true, &mut self.buffer_barriers).unwrap();
            dst_handle = src_handle;
        } else {
            src_handle = self.buffer_states.update_state(op.src_buffer, true, false, &mut self.buffer_barriers).unwrap();
            dst_handle = self.buffer_states.update_state(op.dst_buffer, false, true, &mut self.buffer_barriers).unwrap();
        }

        self.flush_barriers(cmd);

        let ranges: Box<[_]> = op.ranges.as_slice().iter().map(|range| {
            vk::BufferCopy::builder()
                .src_offset(range.src_offset)
                .dst_offset(range.dst_offset)
                .size(range.size)
                .build()
        }).collect();

        unsafe {
            self.vk.cmd_copy_buffer(cmd, src_handle, dst_handle, ranges.as_ref())
        };

        self.has_commands = true;
    }

    fn process_buffer_to_image_transfer(&mut self, op: &BufferToImageTransfer, cmd: vk::CommandBuffer) {
        let buffer_handle = self.buffer_states.update_state(op.src_buffer, true, false, &mut self.buffer_barriers).unwrap();
        let image_handle = self.image_states.update_state_write(op.dst_image, &mut self.image_barriers).unwrap();

        self.flush_barriers(cmd);

        let ranges: Box<[_]> = op.ranges.as_slice().iter().map(|range| {
            vk::BufferImageCopy::builder()
                .buffer_offset(range.buffer_offset)
                .buffer_row_length(range.buffer_row_length)
                .buffer_image_height(range.buffer_image_height)
                .image_subresource(vk::ImageSubresourceLayers {
                    aspect_mask: range.image_aspect_mask,
                    mip_level: range.image_mip_level,
                    base_array_layer: range.image_base_array_layer,
                    layer_count: range.image_layer_count
                })
                .image_offset(vk::Offset3D {
                    x: range.image_offset[0],
                    y: range.image_offset[1],
                    z: range.image_offset[2]
                })
                .image_extent(vk::Extent3D {
                    width: range.image_extent[0],
                    height: range.image_extent[1],
                    depth: range.image_extent[2]
                })
                .build()
        }).collect();

        unsafe {
            self.vk.cmd_copy_buffer_to_image(cmd, buffer_handle, image_handle, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &ranges);
        }

        self.has_commands = true;
    }

    fn record_wait_ops(&mut self, ops: &[SemaphoreOp]) {
        for op in ops {
            let mut info = vk::SemaphoreSubmitInfo::builder()
                .semaphore(op.semaphore)
                .stage_mask(vk::PipelineStageFlags2::TRANSFER);

            if let Some(value) = op.value {
                info = info.value(value);
            }

            self.wait_ops.push(info.build());
        }
    }

    fn record_signal_ops(&mut self, ops: &[SemaphoreOp]) {
        for op in ops {
            let mut info = vk::SemaphoreSubmitInfo::builder()
                .semaphore(op.semaphore)
                .stage_mask(vk::PipelineStageFlags2::TRANSFER);

            if let Some(value) = op.value {
                info = info.value(value);
            }

            self.signal_ops.push(info.build());
        }
    }

    fn flush_barriers(&mut self, cmd: vk::CommandBuffer) {
        if self.buffer_barriers.is_empty() && self.image_barriers.is_empty() {
            return;
        }

        let info = vk::DependencyInfo::builder()
            .buffer_memory_barriers(self.buffer_barriers.as_slice())
            .image_memory_barriers(self.image_barriers.as_slice());

        unsafe {
            self.vk.cmd_pipeline_barrier2(cmd, &info)
        };

        self.buffer_barriers.clear();
        self.image_barriers.clear();

        self.has_commands = true;
    }

    fn submit(&mut self) {
        if !self.has_commands {
            // Nothing has been recorded yet
            return;
        }

        if let Some(cmd) = self.current_buffer.take() {
            self.flush_barriers(cmd);

            unsafe {
                self.vk.end_command_buffer(cmd)
            }.expect("Failed to end command buffer recording");

            self.wait_ops.push(vk::SemaphoreSubmitInfo::builder()
                .semaphore(self.share.semaphore)
                .value(self.last_task_id)
                .stage_mask(vk::PipelineStageFlags2::TRANSFER)
                .build()
            );

            self.signal_ops.push(vk::SemaphoreSubmitInfo::builder()
                .semaphore(self.share.semaphore)
                .value(self.current_task_id)
                .stage_mask(vk::PipelineStageFlags2::TRANSFER)
                .build()
            );

            let submit_info = vk::CommandBufferSubmitInfo::builder()
                .command_buffer(cmd);

            let info = vk::SubmitInfo2::builder()
                .wait_semaphore_infos(self.wait_ops.as_slice())
                .command_buffer_infos(std::slice::from_ref(&submit_info))
                .signal_semaphore_infos(self.signal_ops.as_slice());

            unsafe {
                let guard = self.queue.lock_queue();
                self.vk.queue_submit2(*guard, std::slice::from_ref(&info), vk::Fence::null())
                    .expect("Failed to submit to queue");
            }
            self.wait_ops.clear();
            self.signal_ops.clear();

            // No need to allocate new memory if theres nothing in the vec
            let free_ops = if self.free_ops.is_empty() {
                Vec::new()
            } else {
                std::mem::replace(&mut self.free_ops, Vec::with_capacity(32))
            };
            self.submitted_buffers.push_back((self.current_task_id, cmd, free_ops));

            self.has_commands = false;
            self.last_task_id = self.current_task_id;
            self.last_start = None;
        }
    }
}

impl Drop for TransferWorker {
    fn drop(&mut self) {
        self.submit();

        while let Some((id, _, free)) = self.submitted_buffers.pop_front() {
            let info = vk::SemaphoreWaitInfo::builder()
                .semaphores(std::slice::from_ref(&self.share.semaphore))
                .values(std::slice::from_ref(&id));

            unsafe {
                self.device.vk().wait_semaphores(&info, u64::MAX)
            }.unwrap();

            self.process_free_ops(free);
        }

        let free_ops = std::mem::replace(&mut self.free_ops, Vec::new());
        self.process_free_ops(free_ops);

        unsafe {
            self.device.vk().destroy_command_pool(self.command_pool, None);
        }
    }
}