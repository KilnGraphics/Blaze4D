use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::AtomicU64;

use crate::renderer::emulator::immediate::BufferPool;
use crate::renderer::emulator::descriptors::DescriptorPool;
use crate::renderer::emulator::global_objects::{GlobalObjects, StaticMeshDrawInfo};
use crate::renderer::emulator::{MeshData, PassId, StaticMeshId};
use crate::renderer::emulator::worker::{Channel, WorkerTask};
use crate::renderer::emulator::mc_shaders::{McUniform, Shader, ShaderId, VertexFormat};

use crate::prelude::*;

pub(super) struct Share {
    id: UUID,
    device: Arc<DeviceContext>,
    current_pass: AtomicU64,

    global_objects: GlobalObjects,
    shader_database: Mutex<HashMap<ShaderId, Arc<Shader>>>,
    descriptors: Mutex<DescriptorPool>,
    pool: Mutex<BufferPool>,
    channel: Mutex<Channel>,
    signal: Condvar,
    family: u32,
}

impl Share {
    const PASS_ID_ACTIVE_BIT: u64 = 1u64 << 63;

    pub(super) fn new(device: Arc<DeviceContext>) -> Self {
        let queue = device.get_main_queue();
        let queue_family = queue.get_queue_family_index();

        let global_objects = GlobalObjects::new(device.clone(), queue.clone());
        let descriptors = Mutex::new(DescriptorPool::new(device.clone()));
        let pool = Mutex::new(BufferPool::new(device.clone()));

        Self {
            id: UUID::new(),
            device,
            current_pass: AtomicU64::new(0),
            global_objects,
            shader_database: Mutex::new(HashMap::new()),
            descriptors,
            pool,
            channel: Mutex::new(Channel::new()),
            signal: Condvar::new(),
            family: queue_family,
        }
    }

    pub(super) fn create_static_mesh(&self, data: &MeshData) -> StaticMeshId {
        self.global_objects.create_static_mesh(data)
    }

    pub(super) fn drop_static_mesh(&self, id: StaticMeshId) {
        self.global_objects.mark_static_mesh(id)
    }

    pub(super) fn inc_static_mesh(&self, id: StaticMeshId) -> StaticMeshDrawInfo {
        self.global_objects.inc_static_mesh(id)
    }

    pub(super) fn dec_static_mesh(&self, id: StaticMeshId) {
        self.global_objects.dec_static_mesh(id)
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

    pub(super) fn get_render_queue_family(&self) -> u32 {
        self.family
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

    /// Called by the worker periodically to update any async state or do cleanup
    pub(super) fn worker_update(&self) {
        self.global_objects.update();
    }
}

impl PartialEq for Share {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl Eq for Share {
}

// TODO this is needed because condvar is not unwind safe can we do better?
impl UnwindSafe for Share {
}

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
