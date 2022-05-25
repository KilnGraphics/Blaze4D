use std::collections::HashMap;
use std::time::Duration;
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
                next_sync_id: 1,
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
        let id = guard.next_sync_id;
        guard.next_sync_id += 1;
        guard.task_queue.push_back(Task::BufferRelease(op, id));
        drop(guard);

        self.worker_condvar.notify_one();
        id
    }

    pub(super) fn push_image_release_task(&self, op: ImageAvailabilityOp) -> u64 {
        let mut guard = self.channel.lock().unwrap();
        let id = guard.next_sync_id;
        guard.next_sync_id += 1;
        guard.task_queue.push_back(Task::ImageRelease(op, id));
        drop(guard);

        self.worker_condvar.notify_one();
        id
    }

    pub(super) fn wait_for_submit(&self, id: u64) {
        let mut guard = self.channel.lock().unwrap();
        loop {
            if guard.last_submitted_id >= id {
                return;
            }
            let (guard2, _) = self.new_submit_condvar.wait_timeout(guard, Duration::from_millis(10)).unwrap();
            guard = guard2;
        }
    }

    pub(super) fn wait_for_complete(&self, id: u64) {
        let semaphore = self.semaphore.get_handle();
        let info = vk::SemaphoreWaitInfo::builder()
            .semaphores(std::slice::from_ref(&semaphore))
            .values(std::slice::from_ref(&id));

        unsafe {
            self.device.vk().wait_semaphores(&info, u64::MAX)
        }.unwrap();
    }

    pub(super) fn get_sync_wait_op(&self, id: u64) -> SemaphoreOp {
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

    fn try_get_next_task(&self, timeout: Duration) -> Result<Option<Task>, ()> {
        let mut guard = self.channel.lock().unwrap();
        if let Some(task) = guard.task_queue.pop_front() {
            return Ok(Some(task));
        }
        if guard.terminate {
            return Err(());
        }

        let (mut guard, _) = self.worker_condvar.wait_timeout(guard, timeout).unwrap();
        Ok(guard.task_queue.pop_front())
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
    next_sync_id: u64,
    terminate: bool,
}

#[derive(Debug)]
pub(super) enum Task {
    Flush(u64),
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
    let mut recorder = Recorder::new(share.device.clone(), queue);

    let mut buffers: HashMap<UUID, (BufferState, Option<PoolAllocationId>)> = HashMap::new();

    loop {
        let frees = recorder.process_submitted(share.semaphore.get_handle());
        for free in frees {
            share.allocator.lock().unwrap().free(free);
        }

        let task = share.try_get_next_task(Duration::from_millis(10));
        let task = match task {
            Ok(task) => task,
            Err(_) => break,
        };

        if let Some(task) = task {
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
                    if let Some(barrier) = acquire.get_barrier() {
                        recorder.get_buffer_barriers().push(*barrier);
                    }
                    buffers.insert(acquire.get_buffer().get_id().as_uuid(), (BufferState::new(acquire.buffer, 0, vk::WHOLE_SIZE), None));
                }

                Task::BufferRelease(release, id) => {
                    buffers.remove(&release.buffer.get_id());
                    if let Some(barrier) = release.get_barrier() {
                        recorder.get_buffer_barriers().push(*barrier);
                    }
                    recorder.push_sync(id);
                }

                Task::ImageAcquire(_, _) => {}
                Task::ImageRelease(_, _) => {}

                Task::StagingAcquire(alloc_id, id, buffer, offset, size) => {
                    buffers.insert(id, (BufferState::new(Buffer::from_raw(BufferId::from_raw(id), buffer), offset, size), Some(alloc_id)));
                }

                Task::StagingRelease(id) => {
                    let (_, alloc_id) = buffers.remove(&id).unwrap();
                    if let Some(alloc_id) = alloc_id {
                        recorder.push_free(alloc_id);
                    } else {
                        panic!()
                    }
                }

                Task::BufferTransfer(transfer) => {
                    let mut info = vk::CopyBufferInfo2::builder();

                    if transfer.src_buffer == transfer.dst_buffer {
                        let (buffer, _) = buffers.get_mut(&transfer.src_buffer.as_uuid()).unwrap();
                        buffer.update_state(true, true, recorder.get_buffer_barriers());
                        info = info.src_buffer(buffer.get_handle()).dst_buffer(buffer.get_handle());

                    } else {
                        let (src, _) = buffers.get_mut(&transfer.src_buffer.as_uuid()).unwrap();
                        src.update_state(true, false, recorder.get_buffer_barriers());
                        info = info.src_buffer(src.get_handle());

                        let (dst, _) = buffers.get_mut(&transfer.dst_buffer.as_uuid()).unwrap();
                        dst.update_state(false, true, recorder.get_buffer_barriers());
                        info = info.dst_buffer(dst.get_handle());
                    }

                    let mut copy_regions = Vec::with_capacity(transfer.ranges.as_slice().len());
                    for region in transfer.ranges.as_slice() {
                        copy_regions.push(vk::BufferCopy2::builder()
                            .src_offset(region.src_offset)
                            .dst_offset(region.dst_offset)
                            .size(region.size)
                            .build()
                        );
                    }

                    info = info.regions(copy_regions.as_slice());

                    recorder.flush_barriers();

                    unsafe {
                        share.device.vk().cmd_copy_buffer2(recorder.get_command_buffer(), &info)
                    };
                }

                Task::BufferToImageTransfer(_) => {}
                Task::ImageToBufferTransfer(_) => {}
            }
        }
    }
}