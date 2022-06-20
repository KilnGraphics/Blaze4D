use std::collections::HashMap;
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::time::{Duration, Instant};
use crate::device::transfer::allocator::{PoolAllocation, PoolAllocationId, PoolAllocator};
use crate::device::transfer::recorder::Recorder;

use crate::device::transfer::resource_state::BufferState;
use crate::objects::id::{BufferId, ObjectId};
use crate::objects::sync::{Semaphore, SemaphoreOp, SemaphoreOps};

use super::*;

pub(super) struct Share {
    allocator: Mutex<PoolAllocator>,

    channel: Mutex<Channel>,
    /// Signaled when the [`Channel::last_submitted_id`] has updated.
    new_submit_condvar: Condvar,
    /// Signaled when some new data is available for the worker.
    worker_condvar: Condvar,

    device: Arc<DeviceFunctions>,
    semaphore: Semaphore,
}

// TODO this is needed because condvar is not unwind safe can we do better?
impl UnwindSafe for Share {}

impl RefUnwindSafe for Share {}

impl Share {
    pub(super) fn new(device: Arc<DeviceFunctions>, allocator: Arc<Allocator>) -> Self {
        let mut type_info = vk::SemaphoreTypeCreateInfo::builder()
            .semaphore_type(vk::SemaphoreType::TIMELINE)
            .initial_value(0);

        let info = vk::SemaphoreCreateInfo::builder()
            .push_next(&mut type_info);

        let semaphore = unsafe {
            device.vk.create_semaphore(&info, None)
        }.unwrap();

        Self {
            allocator: Mutex::new(PoolAllocator::new(
                device.clone(),
                allocator,
            )),
            channel: Mutex::new(Channel {
                task_queue: VecDeque::with_capacity(32),
                last_submitted_id: 0,
                next_sync_id: 1,
                terminate: false,
            }),
            new_submit_condvar: Condvar::new(),
            worker_condvar: Condvar::new(),

            device,
            semaphore: Semaphore::new(semaphore),
        }
    }

    /// Allocates some staging memory and makes the memory visible to the worker thread.
    pub(super) fn allocate_staging(&self, min_size: vk::DeviceSize) -> (UUID, PoolAllocation) {
        let mut guard = self.allocator.lock().unwrap_or_else(|_| {
            log::error!("Poisoned allocator mutex in Share::allocate_staging!");
            panic!()
        });

        let staging = guard.allocate(min_size);
        drop(guard);

        let id = UUID::new();
        self.push_task(Task::StagingAcquire(staging.get_id(), id, staging.get_buffer().get_handle(), staging.get_offset(), staging.get_size()));

        (id, staging)
    }

    /// Pushes a tasks to the queue.
    pub(super) fn push_task(&self, task: Task) {
        let mut guard = self.channel.lock().unwrap_or_else(|_| {
            log::error!("Poisoned channel mutex in Share::push_task!");
            panic!()
        });

        guard.task_queue.push_back(task);
        self.worker_condvar.notify_one();
        drop(guard);
    }

    /// Pushes a buffer release tasks to the queue and returns its sync id.
    pub(super) fn push_buffer_release_task(&self, op: BufferReleaseOp) -> u64 {
        let mut guard = self.channel.lock().unwrap_or_else(|_| {
            log::error!("Poisoned channel mutex in Share::push_buffer_release_task!");
            panic!()
        });

        let id = guard.next_sync_id;
        guard.next_sync_id += 1;
        guard.task_queue.push_back(Task::BufferRelease(op, id));
        self.worker_condvar.notify_one();
        drop(guard);

        id
    }

    /// Pushes a image release tasks to the queue and returns its sync id.
    pub(super) fn push_image_release_task(&self, op: ImageReleaseOp) -> u64 {
        let mut guard = self.channel.lock().unwrap_or_else(|_| {
            log::error!("Poisoned channel mutex in Share::push_image_release_task!");
            panic!()
        });

        let id = guard.next_sync_id;
        guard.next_sync_id += 1;
        guard.task_queue.push_back(Task::ImageRelease(op, id));
        self.worker_condvar.notify_one();
        drop(guard);

        id
    }

    /// Waits until all tasks until and including the specified sync id have been submitted for
    /// execution on the queue.
    pub(super) fn wait_for_submit(&self, id: u64) {
        let mut guard = self.channel.lock().unwrap();
        loop {
            if guard.last_submitted_id >= id {
                return;
            }
            let (guard2, timeout) = self.new_submit_condvar.wait_timeout(guard, Duration::from_millis(1000))
                .unwrap_or_else(|_| {
                    log::error!("Poisoned channel mutex in Share::wait_for_submit!");
                    panic!()
                });
            guard = guard2;

            if timeout.timed_out() {
                log::warn!("1s timeout hit in Share::wait_for_submit");
            }
        }
    }

    /// Waits until all tasks until and including the specified sync id have completed execution on
    /// the device.
    pub(super) fn wait_for_complete(&self, id: u64) {
        let semaphore = self.semaphore.get_handle();
        let info = vk::SemaphoreWaitInfo::builder()
            .semaphores(std::slice::from_ref(&semaphore))
            .values(std::slice::from_ref(&id));

        loop {
            match unsafe { self.device.timeline_semaphore_khr.wait_semaphores(&info, 1000000) } {
                Ok(_) => return,
                Err(vk::Result::TIMEOUT) => {
                    log::warn!("1s timeout hit in Share::wait_for_complete");
                },
                Err(err) => {
                    log::error!("vkWaitSemaphores returned {:?} in Share::wait_for_complete", err);
                    panic!()
                }
            }
        }
    }

    /// Generates a semaphore wait operation waiting for completion of the specified sync id.
    pub(super) fn get_sync_wait_op(&self, id: u64) -> SemaphoreOp {
        SemaphoreOp::new_timeline(self.semaphore, id)
    }

    /// Sets the terminate flag notifying the worker thread that it should complete all pending tasks
    /// and shut down.
    ///
    /// Any tasks in the queue will be executed before terminating but any tasks pushed into the
    /// queue after this function is called may not be executed.
    pub(super) fn terminate(&self) {
        let mut guard = self.channel.lock()
            .unwrap_or_else(|_| {
                log::error!("Poisoned channel mutex in Share::terminate!");
                panic!()
            });

        guard.terminate = true;
        self.worker_condvar.notify_all();
    }

    /// Attempts to retrieve a task from the queue, blocking if necessary until the specified
    /// timeout runs out.
    fn try_get_next_task_timeout(&self, timeout: Duration) -> NextTaskResult {
        let start = Instant::now();

        let mut guard = self.channel.lock()
            .unwrap_or_else(|_| {
                log::error!("Poisoned channel mutex in Share::try_get_next_task!");
                panic!()
            });

        loop {
            if let Some(task) = guard.task_queue.pop_front() {
                return NextTaskResult::Ok(task);
            }
            if guard.terminate {
                // We only terminate after the queue is empty and the terminate signal has been sent
                return NextTaskResult::Terminate;
            }

            let diff = (start + timeout).saturating_duration_since(Instant::now());
            if diff.is_zero() {
                return NextTaskResult::Timeout;
            }

            let (new_guard, timeout) = self.worker_condvar.wait_timeout(guard, diff)
                .unwrap_or_else(|_| {
                    log::error!("Poisoned channel mutex in Share::try_get_next_task!");
                    panic!()
                });
            guard = new_guard;

            if timeout.timed_out() {
                return NextTaskResult::Timeout;
            }
        }
    }
}

enum NextTaskResult {
    Ok(Task),
    Timeout,
    Terminate,
}

impl Drop for Share {
    fn drop(&mut self) {
        unsafe {
            self.device.vk.destroy_semaphore(self.semaphore.get_handle(), None);
        }
    }
}

pub(super) struct Channel {
    task_queue: VecDeque<Task>,
    last_submitted_id: u64,
    next_sync_id: u64,
    terminate: bool,
}

#[derive(Debug)]
pub(super) enum Task {
    Flush(u64),
    BufferAcquire(BufferAcquireOp, SemaphoreOps),
    BufferRelease(BufferReleaseOp, u64),
    ImageAcquire(ImageAcquireOp, SemaphoreOps),
    ImageRelease(ImageReleaseOp, u64),
    StagingAcquire(PoolAllocationId, UUID, vk::Buffer, vk::DeviceSize, vk::DeviceSize),
    StagingRelease(UUID),
    BufferTransfer(BufferTransfer),
    BufferToImageTransfer(BufferToImageTransfer),
    ImageToBufferTransfer(ImageToBufferTransfer),
}

pub(super) fn run_worker(share: Arc<Share>, queue: Arc<Queue>) {
    let mut recorder = Recorder::new(share.device.clone(), queue);

    let mut buffers: HashMap<UUID, (BufferState, Option<PoolAllocationId>)> = HashMap::new();

    loop {
        let frees = recorder.process_submitted(share.semaphore.get_handle());
        for free in frees {
            share.allocator.lock().unwrap().free(free);
        }

        let task = share.try_get_next_task_timeout(Duration::from_millis(10));
        let task = match task {
            NextTaskResult::Ok(task) => task,
            NextTaskResult::Timeout => continue,
            NextTaskResult::Terminate => break,
        };

        match task {
            Task::Flush(_) => {
                let sync_id = recorder.submit(share.semaphore.get_handle());
                if let Some(id) = sync_id {
                    share.channel.lock().unwrap().last_submitted_id = id;
                    share.new_submit_condvar.notify_all();
                }
            }

            Task::BufferAcquire(acquire, waits) => {
                recorder.add_wait_ops(waits);
                if let Some(barrier) = acquire.make_transfer_barrier(vk::PipelineStageFlags2::TRANSFER, vk::AccessFlags2::TRANSFER_READ | vk::AccessFlags2::TRANSFER_WRITE) {
                    recorder.get_buffer_barriers().push(barrier);
                }
                if buffers.insert(acquire.get_buffer().get_id().as_uuid(), (BufferState::new(acquire.buffer, 0, vk::WHOLE_SIZE), None)).is_some() {
                    log::error!("Acquired buffer {:?} which was already available!", acquire.buffer);
                    panic!()
                }
            }

            Task::BufferRelease(release, id) => {
                buffers.remove(&release.buffer.get_id()).unwrap_or_else(|| {
                    log::error!("Released buffer {:?} which was not available!", release.buffer);
                    panic!()
                });
                if let Some(barrier) = release.make_transfer_barrier(vk::PipelineStageFlags2::TRANSFER, vk::AccessFlags2::TRANSFER_READ | vk::AccessFlags2::TRANSFER_WRITE) {
                    recorder.get_buffer_barriers().push(barrier);
                }
                recorder.push_sync(id);
            }

            Task::ImageAcquire(_, _) => {}
            Task::ImageRelease(_, _) => {}

            Task::StagingAcquire(alloc_id, id, buffer, offset, size) => {
                if buffers.insert(id, (BufferState::new(Buffer::from_raw(BufferId::from_raw(id), buffer), offset, size), Some(alloc_id))).is_some() {
                    log::error!("Acquired staging buffer {:?} which was already available!", id);
                    panic!()
                }
            }

            Task::StagingRelease(id) => {
                let (_, alloc_id) = buffers.remove(&id).unwrap_or_else(|| {
                    log::error!("Release staging buffer {:?} which was not available!", id);
                    panic!()
                });


                if let Some(alloc_id) = alloc_id {
                    recorder.push_free(alloc_id);
                } else {
                    log::error!("Released staging buffer {:?} which has no allocation id!", id);
                    panic!()
                }
            }

            Task::BufferTransfer(transfer) => {
                let src_buff;
                let dst_buff;
                if transfer.src_buffer == transfer.dst_buffer {
                    let (buffer, _) = buffers.get_mut(&transfer.src_buffer.as_uuid()).unwrap_or_else(|| {
                        log::error!("Transfer buffer {:?} is not available!", transfer.src_buffer);
                        panic!()
                    });
                    buffer.update_state(true, true, recorder.get_buffer_barriers());
                    src_buff = buffer.get_handle();
                    dst_buff = buffer.get_handle();

                } else {
                    let (src, _) = buffers.get_mut(&transfer.src_buffer.as_uuid()).unwrap_or_else(|| {
                        log::error!("Transfer src buffer {:?} is not available!", transfer.src_buffer);
                        panic!()
                    });
                    src.update_state(true, false, recorder.get_buffer_barriers());
                    src_buff = src.get_handle();

                    let (dst, _) = buffers.get_mut(&transfer.dst_buffer.as_uuid()).unwrap_or_else(|| {
                        log::error!("Transfer dst buffer {:?} is not available!", transfer.src_buffer);
                        panic!()
                    });
                    dst.update_state(false, true, recorder.get_buffer_barriers());
                    dst_buff = dst.get_handle();
                }

                let mut copy_regions = Vec::with_capacity(transfer.ranges.as_slice().len());
                for region in transfer.ranges.as_slice() {
                    copy_regions.push(vk::BufferCopy::builder()
                        .src_offset(region.src_offset)
                        .dst_offset(region.dst_offset)
                        .size(region.size)
                        .build()
                    );
                }

                recorder.flush_barriers();

                unsafe {
                    share.device.vk.cmd_copy_buffer(recorder.get_command_buffer(), src_buff, dst_buff, copy_regions.as_slice())
                };
            }

            Task::BufferToImageTransfer(_) => {}
            Task::ImageToBufferTransfer(_) => {}
        }
    }
}