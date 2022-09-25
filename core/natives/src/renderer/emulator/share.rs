use std::any::Any;
use std::sync::{Arc, Condvar, LockResult, Mutex, MutexGuard};
use std::time::{Duration, Instant};
use std::panic::RefUnwindSafe;
use std::collections::{HashMap, VecDeque};
use std::fmt::{Debug, Display, Formatter};
use std::ops::Add;
use std::os::unix::raw::gid_t;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::thread::JoinHandle;
use ash::vk;
use crate::allocator::{Allocation, HostAccess};

use crate::renderer::emulator::descriptors::DescriptorPool;
use crate::renderer::emulator::worker::{EmulatorTaskContainer, run_worker2, WorkerTask, WorkerTask3};
use crate::renderer::emulator::mc_shaders::{McUniform, Shader, ShaderId, VertexFormat};
use crate::renderer::emulator::immediate::{ImmediateBuffer, ImmediatePool};
use crate::renderer::emulator::staging::{StagingAllocationId2, StagingAllocation2, StagingMemory2, StagingMemoryPool};

use crate::prelude::*;
use crate::renderer::emulator::{ExportHandle, ExportSet};


pub(super) struct Share2 {
    device: Arc<DeviceContext>,
    queue: Arc<Queue>,

    id: UUID,

    staging: Mutex<StagingMemory2>,
    channel: Mutex<Channel2>,
    semaphore: vk::Semaphore,
    signal: Condvar,
}

impl Share2 {
    pub(super) fn new(device: Arc<DeviceContext>, queue: Arc<Queue>) -> (Arc<Self>, JoinHandle<()>) {
        let staging = StagingMemory2::new(device.clone());

        let mut type_info = vk::SemaphoreTypeCreateInfo::builder()
            .semaphore_type(vk::SemaphoreType::TIMELINE_KHR)
            .initial_value(0);

        let info = vk::SemaphoreCreateInfo::builder()
            .push_next(&mut type_info);

        let semaphore = unsafe {
            device.vk().create_semaphore(&info, None)
        }.expect("Failed to create semaphore for emulator");

        let share = Arc::new(Self {
            device,
            queue,
            id: UUID::new(),
            staging: Mutex::new(staging),
            channel: Mutex::new(Channel2::new()),
            semaphore,
            signal: Condvar::new(),
        });

        let share_clone = share.clone();
        let worker = std::thread::spawn(move || {
            let share = share_clone.clone();
            if let Err(err) = std::panic::catch_unwind(move || {
                run_worker2(share);
                log::debug!("Emulator worker thread finished");
            }) {
                let err_ref: &dyn Any = &err;
                if let Some(err) = err_ref.downcast_ref::<&dyn Debug>() {
                    log::error!("Emulator worker thread panicked: {:?}", err);
                } else {
                    log::error!("Emulator worker thread panicked with non debug error");
                }
                if let Ok(mut guard) = share_clone.channel.lock() {
                    guard.state = State::Failed;
                    guard.queue.clear(); // Need to make sure we dont have any cyclic Arc's
                } else {
                    log::warn!("Failed to set failed flag after emulator worker thread panicked");
                }
                panic!("Emulator worker thread panicked");
            }
        });

        (share, worker)
    }

    pub(super) fn get_device(&self) -> &Arc<DeviceContext> {
        &self.device
    }

    pub(super) fn get_queue(&self) -> &Arc<Queue> {
        &self.queue
    }

    pub(super) fn get_semaphore(&self) -> vk::Semaphore {
        self.semaphore
    }

    pub(super) fn wait_for_task(&self, task_id: u64) {
        let info = vk::SemaphoreWaitInfo::builder()
            .semaphores(std::slice::from_ref(&self.semaphore))
            .values(std::slice::from_ref(&task_id));

        loop {
            match unsafe {
                self.device.timeline_semaphore_khr().wait_semaphores(&info, 1000000000)
            } {
                Ok(()) => break,
                Err(vk::Result::TIMEOUT) => {
                    log::warn!("Timeout while waiting on emulator semaphore");
                    if self.channel.lock().unwrap().state == State::Failed {
                        panic!("Emulator worker has failed");
                    }
                },
                Err(err) => panic!("VkWaitSemaphores returned {:?}", err),
            }
        }
    }

    pub(super) fn wait_for_export(&self, export: u64) {
        let mut guard = self.channel.lock().unwrap();
        loop {
            if guard.export_ready >= export {
                return;
            }
            let (new_guard, result) = self.signal.wait_timeout(guard, Duration::from_secs(1)).unwrap();
            guard = new_guard;

            if result.timed_out() {
                log::warn!("Timeout while waiting for result ready");
                if guard.state == State::Failed {
                    panic!("Emulator worker has failed");
                }
            }
        }
    }

    pub(super) fn signal_export(&self, export: u64) {
        let mut guard = self.channel.lock().unwrap();
        guard.export_ready = export;
        drop(guard);
        self.signal.notify_all();
    }

    pub(super) fn allocate_staging(&self, size: u64, alignment: u64) -> (StagingAllocation2, StagingAllocationId2) {
        self.staging.lock().unwrap().allocate(size, alignment)
    }

    pub(super) unsafe fn free_staging<I: IntoIterator<Item=StagingAllocationId2>>(&self, iter: I) {
        let mut guard = self.staging.lock().unwrap();
        for i in iter.into_iter() {
            guard.free(i);
        }
    }

    pub(super) fn push_task(&self, task: EmulatorTaskContainer) -> u64 {
        let mut guard = self.channel.lock().unwrap();
        if guard.state != State::Running {
            panic!("Called push_task on {:?} share", guard.state);
        }

        let id = guard.next_task_id;
        guard.queue.push_back(WorkerTask3::Emulator(task));

        drop(guard);
        self.signal.notify_all();
        id
    }

    pub(super) fn export(&self, export_set: Arc<ExportSet>) -> ExportHandle {
        let mut guard = self.channel.lock().unwrap();
        if guard.state != State::Running {
            panic!("Called export on {:?} share", guard.state);
        }

        let emulator_signal_value = guard.next_task_id;
        guard.next_task_id += 1;
        let export_signal_value = guard.next_task_id;
        guard.next_task_id += 1;

        guard.queue.push_back(WorkerTask3::Export(emulator_signal_value, export_signal_value, export_set.clone()));

        drop(guard);
        self.signal.notify_all();
        ExportHandle {
            export_set,
            wait_value: emulator_signal_value,
            signal_value: export_signal_value,
        }
    }

    pub(super) fn flush(&self) -> u64 {
        let mut guard = self.channel.lock().unwrap();
        if guard.state != State::Running {
            panic!("Called flush on {:?} share", guard.state);
        }

        let id = guard.next_task_id;
        guard.next_task_id += 1;

        guard.queue.push_back(WorkerTask3::Flush(id));

        drop(guard);
        self.signal.notify_all();
        id
    }

    pub(super) fn shutdown(&self) {
        let mut guard = self.channel.lock().unwrap();
        if guard.state != State::Running {
            panic!("Called shutdown on {:?} share", guard.state);
        }

        let id = guard.next_task_id;
        guard.queue.push_back(WorkerTask3::Shutdown(id));
        guard.state = State::Shutdown;
        drop(guard);
        self.signal.notify_all();
    }

    pub(super) fn pop_task(&self, timeout: Duration) -> Option<WorkerTask3> {
        let mut guard = self.channel.lock().unwrap();
        // On shutdown the worker first has to finish all pending tasks
        if guard.state == State::Failed {
            panic!("Share state is failed");
        }

        if let Some(task) = guard.queue.pop_front() {
            Some(task)
        } else {
            let (mut guard, _) = self.signal.wait_timeout_while(guard, timeout, |g| g.queue.is_empty()).unwrap();
            guard.queue.pop_front()
        }
    }

    pub(super) fn update(&self) {
        self.staging.lock().unwrap().update();
    }

    /// Ensures all self references are destroyed. Should be be called by the emulator after waiting
    /// for the worker to finish execution to ensure all resources are freed.
    pub(super) fn cleanup(&self) {
        match self.channel.lock() {
            Ok(mut guard) => {
                if guard.state == State::Running {
                    drop(guard);
                    panic!("Called cleanup on still running emulator");
                }
                guard.queue.clear()
            },
            Err(mut err) => err.get_mut().queue.clear(),
        }
    }
}

impl Drop for Share2 {
    fn drop(&mut self) {
        unsafe {
            self.device.vk().destroy_semaphore(self.semaphore, None);
        }
    }
}

impl PartialEq for Share2 {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Share2 {
}

// Condvar issues
impl RefUnwindSafe for Share2 {
}

struct Channel2 {
    queue: VecDeque<WorkerTask3>,
    next_task_id: u64,
    export_ready: u64,
    state: State,
}

impl Channel2 {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            next_task_id: 1,
            export_ready: 0,
            state: State::Running
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
enum State {
    Running,
    Failed,
    Shutdown,
}









pub(super) struct Share {
    id: UUID,
    device: Arc<DeviceContext>,
    current_pass: AtomicU64,

    staging_memory: Mutex<StagingMemoryPool>,
    immediate_buffers: ImmediatePool,
    shader_database: Mutex<HashMap<ShaderId, Arc<Shader>>>,
    descriptors: Mutex<DescriptorPool>,
    channel: Mutex<Channel>,
    signal: Condvar,
}

impl Share {
    const PASS_ID_ACTIVE_BIT: u64 = 1u64 << 63;

    pub(super) fn new(device: Arc<DeviceContext>) -> Self {
        let queue = device.get_main_queue();

        let staging_memory = StagingMemoryPool::new(device.clone());
        let immediate_buffers = ImmediatePool::new(device.clone());
        let descriptors = Mutex::new(DescriptorPool::new(device.clone()));

        Self {
            id: UUID::new(),
            device,
            current_pass: AtomicU64::new(0),

            staging_memory: Mutex::new(staging_memory),
            immediate_buffers,
            shader_database: Mutex::new(HashMap::new()),
            descriptors,
            channel: Mutex::new(Channel::new()),
            signal: Condvar::new(),
        }
    }

    pub(super) fn get_device(&self) -> &Arc<DeviceContext> {
        &self.device
    }

    pub(super) fn get_staging_pool(&self) -> &Mutex<StagingMemoryPool> {
        &self.staging_memory
    }

    pub(super) fn create_shader(&self, vertex_format: &VertexFormat, used_uniforms: McUniform) -> ShaderId {
        let shader = Shader::new(*vertex_format, used_uniforms);
        let id = shader.get_id();

        let mut guard = self.shader_database.lock().unwrap();
        guard.insert(id, shader);

        id
    }

    pub(super) fn drop_shader(&self, id: ShaderId) {
        let mut guard = self.shader_database.lock().unwrap();
        guard.remove(&id);
    }

    pub(super) fn get_shader(&self, id: ShaderId) -> Option<Arc<Shader>> {
        let guard = self.shader_database.lock().unwrap();
        guard.get(&id).cloned()
    }

    pub(super) fn get_current_pass_id(&self) -> Option<u64> {
        let id = self.current_pass.load(std::sync::atomic::Ordering::Acquire);
        if (id & Self::PASS_ID_ACTIVE_BIT) == Self::PASS_ID_ACTIVE_BIT {
            Some(id & !Self::PASS_ID_ACTIVE_BIT)
        } else {
            None
        }
    }

    pub(super) fn try_start_pass_id(&self) -> Option<u64> {
        loop {
            let old_id = self.current_pass.load(std::sync::atomic::Ordering::Acquire);
            if (old_id & Self::PASS_ID_ACTIVE_BIT) == Self::PASS_ID_ACTIVE_BIT {
                return None;
            }
            let new_id = old_id + 1;
            if (new_id & Self::PASS_ID_ACTIVE_BIT) == Self::PASS_ID_ACTIVE_BIT {
                log::error!("Pass id overflow. This is either a bug or this application has been running for a few thousand years");
                panic!()
            }
            if let Ok(_) = self.current_pass.compare_exchange(
                old_id, new_id | Self::PASS_ID_ACTIVE_BIT,
                std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::Acquire
            ) {
                return Some(new_id);
            }
        }
    }

    pub(super) fn end_pass_id(&self) {
        let old_id = self.current_pass.load(std::sync::atomic::Ordering::Acquire);
        if (old_id & Self::PASS_ID_ACTIVE_BIT) == 0 {
            log::error!("Called Share::end_pass_id with no active pass!");
            panic!()
        }
        let new_id = old_id & !Self::PASS_ID_ACTIVE_BIT;
        self.current_pass.compare_exchange(old_id, new_id, std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::Acquire).unwrap_or_else(|_| {
            log::error!("Current pass id has been modified while Share::end_pass_id is running!");
            panic!();
        });
    }

    pub(super) fn get_next_immediate_buffer(&self) -> Box<ImmediateBuffer> {
        self.immediate_buffers.get_next_buffer()
    }

    pub(super) fn return_immediate_buffer(&self, buffer: Box<ImmediateBuffer>) {
        self.immediate_buffers.return_buffer(buffer);
    }

    pub(super) fn allocate_uniform(&self, data: &[u8]) -> (vk::Buffer, vk::DeviceSize) {
        self.descriptors.lock().unwrap().allocate_uniform(data)
    }

    pub(super) fn push_task(&self, task: WorkerTask) {
        self.channel.lock().unwrap().queue.push_back(task);
        self.signal.notify_one();
    }

    pub(super) fn try_get_next_task_timeout(&self, timeout: Duration) -> NextTaskResult {
        let start = Instant::now();

        let mut guard = self.channel.lock().unwrap_or_else(|_| {
            log::error!("Poisoned channel mutex in Share::try_get_next_task!");
            panic!()
        });

        loop {
            if let Some(task) = guard.queue.pop_front() {
                return NextTaskResult::Ok(task);
            }

            let diff = (start + timeout).saturating_duration_since(Instant::now());
            if diff.is_zero() {
                return NextTaskResult::Timeout;
            }

            let (new_guard, timeout) = self.signal.wait_timeout(guard, diff).unwrap_or_else(|_| {
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

impl PartialEq for Share {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl Eq for Share {
}

// Condvar issues
impl RefUnwindSafe for Share {
}

pub(in crate::renderer::emulator) enum NextTaskResult {
    Ok(WorkerTask),
    Timeout,
}

struct Channel {
    queue: VecDeque<WorkerTask>,
}

impl Channel {
    fn new() -> Self {
        Self {
            queue: VecDeque::new()
        }
    }
}
