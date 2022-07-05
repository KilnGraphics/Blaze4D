use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};
use std::panic::RefUnwindSafe;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::AtomicU64;
use ash::vk;

use crate::renderer::emulator::descriptors::DescriptorPool;
use crate::renderer::emulator::worker::WorkerTask;
use crate::renderer::emulator::mc_shaders::{McUniform, Shader, ShaderId, VertexFormat};

use crate::prelude::*;
use crate::renderer::emulator::immediate::{ImmediateBuffer, ImmediatePool};
use crate::renderer::emulator::staging::StagingMemoryPool;

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
