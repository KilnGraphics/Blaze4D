use std::collections::VecDeque;
use std::sync::Arc;

use ash::vk;
use crate::device::device::Queue;
use crate::device::transfer::allocator::PoolAllocationId;
use crate::objects::sync::{SemaphoreOp, SemaphoreOps};

use crate::prelude::DeviceContext;

pub(super) struct Recorder {
    device: Arc<DeviceContext>,
    queue: Queue,

    command_pool: CommandBufferPool,
    submitted_buffers: VecDeque<(Option<u64>, vk::CommandBuffer, Vec<PoolAllocationId>)>,

    cmd: Option<vk::CommandBuffer>,

    max_sync: Option<u64>,
    pool_frees: Vec<PoolAllocationId>,

    wait_ops: Vec<SemaphoreOp>,
    signal_ops: Vec<SemaphoreOp>,

    pending_buffer_barriers: Vec<vk::BufferMemoryBarrier2>,
    pending_image_barriers: Vec<vk::ImageMemoryBarrier2>,
}

impl Recorder {
    pub(super) fn new(device: Arc<DeviceContext>, queue: Queue) -> Self {
        let command_pool = CommandBufferPool::new(device.clone(), queue.get_queue_family_index());

        Self {
            device,
            queue,
            command_pool,
            submitted_buffers: VecDeque::new(),
            cmd: None,
            max_sync: None,
            pool_frees: Vec::new(),
            wait_ops: Vec::new(),
            signal_ops: Vec::new(),
            pending_buffer_barriers: Vec::new(),
            pending_image_barriers: Vec::new(),
        }
    }

    pub(super) fn push_free(&mut self, free: PoolAllocationId) {
        self.pool_frees.push(free);
    }

    pub(super) fn process_submitted(&mut self, release_semaphore: vk::Semaphore) -> Vec<PoolAllocationId> {
        let value = unsafe {
            self.device.vk().get_semaphore_counter_value(release_semaphore)
        }.unwrap();

        let mut frees = Vec::new();
        let mut cache: Vec<(vk::CommandBuffer, Vec<PoolAllocationId>)> = Vec::new();

        while let Some((release, cmd, free)) = self.submitted_buffers.pop_front() {
            if let Some(release) = release {
                if release <= value {
                    for (cmd, free) in cache.iter() {
                        self.command_pool.return_buffer(*cmd);
                        for f in free {
                            frees.push(*f);
                        }
                    }
                    cache.clear();
                    self.command_pool.return_buffer(cmd);
                    frees.extend(free);
                } else {
                    self.submitted_buffers.push_front((Some(release), cmd, free));
                    break;
                }
            } else {
                cache.push((cmd, free));
            }
        }

        for (cmd, free) in cache {
            self.submitted_buffers.push_front((None, cmd, free));
        }

        frees
    }

    pub(super) fn get_command_buffer(&mut self) -> vk::CommandBuffer {
        if let Some(cmd) = self.cmd {
            cmd
        } else {
            let cmd = self.command_pool.get_buffer();

            let info = vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

            unsafe {
                self.device.vk().begin_command_buffer(cmd, &info)
            }.unwrap();

            self.cmd = Some(cmd);
            cmd
        }
    }

    pub(super) fn push_sync(&mut self, sync_id: u64) {
        self.max_sync = Some(self.max_sync.map_or(sync_id, |max| std::cmp::max(max, sync_id)));
    }

    pub(super) fn add_wait_ops(&mut self, wait_ops: SemaphoreOps) {
        self.wait_ops.extend(wait_ops.as_slice());
    }

    pub(super) fn get_buffer_barriers(&mut self) -> &mut Vec<vk::BufferMemoryBarrier2> {
        &mut self.pending_buffer_barriers
    }

    pub(super) fn flush_barriers(&mut self) {
        if !self.pending_buffer_barriers.is_empty() || !self.pending_image_barriers.is_empty() {
            let cmd = self.get_command_buffer();

            let info = vk::DependencyInfo::builder()
                .buffer_memory_barriers(self.pending_buffer_barriers.as_slice())
                .image_memory_barriers(self.pending_image_barriers.as_slice());

            unsafe {
                self.device.vk().cmd_pipeline_barrier2(cmd, &info)
            };

            self.pending_buffer_barriers.clear();
            self.pending_image_barriers.clear();
        }
    }

    pub(super) fn submit(&mut self, sync_semaphore: vk::Semaphore) -> Option<u64> {
        self.flush_barriers();

        if let Some(cmd) = self.cmd.take() {
            unsafe {
                self.device.vk().end_command_buffer(cmd)
            }.unwrap();

            let mut wait_info = Vec::with_capacity(self.wait_ops.len());
            for wait_op in &self.wait_ops {
                wait_info.push(vk::SemaphoreSubmitInfo::builder()
                    .stage_mask(vk::PipelineStageFlags2::TRANSFER)
                    .semaphore(wait_op.semaphore.get_handle())
                    .value(wait_op.value.unwrap_or(0))
                    .build()
                );
            }

            let additional_semaphores = self.max_sync.map_or(0, |_| 1);
            let mut signal_info = Vec::with_capacity(self.signal_ops.len() + additional_semaphores);
            for signal_op in &self.signal_ops {
                signal_info.push(vk::SemaphoreSubmitInfo::builder()
                    .stage_mask(vk::PipelineStageFlags2::TRANSFER)
                    .semaphore(signal_op.semaphore.get_handle())
                    .value(signal_op.value.unwrap_or(0))
                    .build()
                );
            }
            if let Some(max_sync) = &self.max_sync {
                signal_info.push(vk::SemaphoreSubmitInfo::builder()
                    .stage_mask(vk::PipelineStageFlags2::TRANSFER)
                    .semaphore(sync_semaphore)
                    .value(*max_sync)
                    .build()
                );
            }

            let cmd_info = vk::CommandBufferSubmitInfo::builder()
                .command_buffer(cmd);

            let info = vk::SubmitInfo2::builder()
                .wait_semaphore_infos(wait_info.as_slice())
                .command_buffer_infos(std::slice::from_ref(&cmd_info))
                .signal_semaphore_infos(signal_info.as_slice());

            unsafe {
                self.queue.submit_2(std::slice::from_ref(&info), None)
            }.unwrap();

            self.submitted_buffers.push_back((self.max_sync, cmd, std::mem::replace(&mut self.pool_frees, Vec::new())));

            self.wait_ops.clear();
            self.signal_ops.clear();
        }

        self.max_sync.take()
    }
}

struct CommandBufferPool {
    device: Arc<DeviceContext>,
    pool: vk::CommandPool,
    buffers: Vec<vk::CommandBuffer>,
}

impl CommandBufferPool {
    fn new(device: Arc<DeviceContext>, queue: u32) -> Self {
        let info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER | vk::CommandPoolCreateFlags::TRANSIENT)
            .queue_family_index(queue);

        let pool = unsafe {
            device.vk().create_command_pool(&info, None)
        }.unwrap();

        Self {
            device,
            pool,
            buffers: Vec::new(),
        }
    }

    fn get_buffer(&mut self) -> vk::CommandBuffer {
        match self.buffers.pop() {
            Some(buffer) => buffer,
            None => {
                self.allocate_buffers();
                self.buffers.pop().unwrap()
            }
        }
    }

    fn return_buffer(&mut self, buffer: vk::CommandBuffer) {
        self.buffers.push(buffer);
    }

    fn allocate_buffers(&mut self) {
        let info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.pool)
            .command_buffer_count(16)
            .level(vk::CommandBufferLevel::PRIMARY);

        let buffers = unsafe {
            self.device.vk().allocate_command_buffers(&info)
        }.unwrap();

        self.buffers.extend(buffers)
    }
}

impl Drop for CommandBufferPool {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().destroy_command_pool(self.pool, None);
        }
    }
}