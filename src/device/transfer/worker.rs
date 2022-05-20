use std::collections::HashMap;
use std::sync::LockResult;
use std::time::{Duration, Instant};
use winit::event::VirtualKeyCode::M;
use crate::device::transfer::allocator::{PoolAllocation, PoolAllocationId, PoolAllocator};

use crate::device::transfer::resource_state::{BufferState, ImageStateTracker};
use crate::objects::id::SemaphoreId;
use crate::objects::sync::{Semaphore, SemaphoreOp, SemaphoreOps};

use super::*;

pub(super) struct Share {
    allocator: Mutex<PoolAllocator>,

    channel: Mutex<Channel>,
    /// Signaled when the [`Channel::last_submitted_id`] has updated.
    new_submit_condvar: Condvar,
    /// Signaled when some new data is available for the worker.
    worker_condvar: Condvar,

    device: Arc<DeviceContext>,
    semaphore: Semaphore,
}

impl Share {
    pub(super) fn new(device: Arc<DeviceContext>, allocator: Arc<Allocator>) -> Self {
        let mut type_info = vk::SemaphoreTypeCreateInfo::builder()
            .semaphore_type(vk::SemaphoreType::TIMELINE)
            .initial_value(0);

        let info = vk::SemaphoreCreateInfo::builder()
            .push_next(&mut type_info);

        let semaphore = unsafe {
            device.vk().create_semaphore(&info, None)
        }.unwrap();

        Self {
            allocator: Mutex::new(PoolAllocator::new(
                device.clone(),
                    allocator,
            )),
            channel: Mutex::new(Channel {
                task_queue: VecDeque::with_capacity(32),
                last_submitted_id: 0,
                next_release_id: 1,
                terminate: false,
            }),
            new_submit_condvar: Condvar::new(),
            worker_condvar: Condvar::new(),

            device,
            semaphore: Semaphore::new(semaphore)
        }
    }

    /// Allocates some staging memory and makes the memory visible to the worker thread.
    pub(super) fn allocate_staging(&self, min_size: vk::DeviceSize) -> (UUID, PoolAllocation) {
        let staging = self.allocator.lock().unwrap().allocate(min_size);
        let id = UUID::new();
        self.push_task(Task::StagingAcquire(staging.get_id(), id, staging.get_buffer().get_handle(), staging.get_offset(), staging.get_size()));

        (id, staging)
    }

    pub(super) fn push_task(&self, task: Task) {
        let mut guard = self.channel.lock().unwrap();
        guard.task_queue.push_back(task);
        drop(guard);

        self.worker_condvar.notify_one();
    }

    pub(super) fn push_buffer_release_task(&self, op: BufferAvailabilityOp) -> u64 {
        let mut guard = self.channel.lock().unwrap();
        let id = guard.next_release_id;
        guard.next_release_id += 1;
        guard.task_queue.push_back(Task::BufferRelease(op, id));
        drop(guard);

        self.worker_condvar.notify_one();
        id
    }

    pub(super) fn push_image_release_task(&self, op: ImageAvailabilityOp) -> u64 {
        let mut guard = self.channel.lock().unwrap();
        let id = guard.next_release_id;
        guard.next_release_id += 1;
        guard.task_queue.push_back(Task::ImageRelease(op, id));
        drop(guard);

        self.worker_condvar.notify_one();
        id
    }

    pub(super) fn wait_for_release_submit(&self, id: u64) {
        let mut guard = self.channel.lock().unwrap();
        loop {
            if guard.last_submitted_id >= id {
                return;
            }
            guard = self.new_submit_condvar.wait(guard).unwrap();
        }
    }

    pub(super) fn get_release_wait_op(&self, id: u64) -> SemaphoreOp {
        SemaphoreOp::new_timeline(self.semaphore, id)
    }

    pub(super) fn terminate(&self) {
        match self.channel.lock() {
            Ok(mut guard) => {
                guard.terminate = true;
            }
            Err(mut err) => {
                log::error!("Transfer channel mutex is poisoned!");
                err.get_mut().terminate = true;
            }
        }
        self.worker_condvar.notify_all();
    }

    fn try_get_next_task(&self, timeout: Duration) -> Option<Task> {
        let mut guard = self.channel.lock().unwrap();
        if let Some(task) = guard.task_queue.pop_front() {
            return Some(task);
        }

        let (mut guard, _) = self.worker_condvar.wait_timeout(guard, timeout).unwrap();
        guard.task_queue.pop_front()
    }
}

impl Drop for Share {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().destroy_semaphore(self.semaphore.get_handle(), None);
        }
    }
}

pub(super) struct Channel {
    task_queue: VecDeque<Task>,
    last_submitted_id: u64,
    next_release_id: u64,
    terminate: bool,
}

#[derive(Debug)]
pub(super) enum Task {
    FlushRelease(u64),
    FlushStaging(UUID),
    BufferAcquire(BufferAvailabilityOp, SemaphoreOps),
    BufferRelease(BufferAvailabilityOp, u64),
    ImageAcquire(ImageAvailabilityOp, SemaphoreOps),
    ImageRelease(ImageAvailabilityOp, u64),
    StagingAcquire(PoolAllocationId, UUID, vk::Buffer, vk::DeviceSize, vk::DeviceSize),
    StagingRelease(UUID),
    BufferTransfer(BufferTransfer),
    BufferToImageTransfer(BufferToImageTransfer),
    ImageToBufferTransfer(ImageToBufferTransfer),
}

pub(super) fn run_worker(share: Arc<Share>, queue: VkQueue) {
}