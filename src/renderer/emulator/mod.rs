mod buffer;
mod worker;
pub mod pipeline;
pub mod debug_pipeline;
pub mod pass;

use std::sync::{Arc, Mutex, Weak};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::renderer::emulator::buffer::BufferPool;
use crate::renderer::emulator::pass::{PassRecorder, PassId};
use crate::renderer::emulator::worker::{run_worker, Share};

use crate::vk::DeviceEnvironment;

use crate::renderer::emulator::pipeline::EmulatorPipeline;

pub(crate) struct EmulatorRenderer {
    weak: Weak<EmulatorRenderer>,
    device: DeviceEnvironment,
    worker: Arc<Share>,
    next_frame_id: AtomicU64,
    buffer_pool: Arc<Mutex<BufferPool>>,
}

impl EmulatorRenderer {
    pub(crate) fn new(device: DeviceEnvironment) -> Arc<Self> {
        let renderer = Arc::new_cyclic(|weak| {
            let pool = Arc::new(Mutex::new(BufferPool::new(device.clone())));

            Self {
                weak: weak.clone(),
                device: device.clone(),
                worker: Arc::new(Share::new(device.clone(), pool.clone())),
                next_frame_id: AtomicU64::new(1),
                buffer_pool: pool,
            }
        });

        let share = renderer.worker.clone();

        std::thread::spawn(move || {
            std::panic::catch_unwind(|| {
                run_worker(device, share);
            }).unwrap_or_else(|_| {
                log::error!("Emulator worker panicked!");
                std::process::exit(1);
            })
        });

        renderer
    }

    pub fn start_pass(&self, pipeline: Arc<dyn EmulatorPipeline>) -> PassRecorder {
        let id = PassId::from_raw(self.next_frame_id.fetch_add(1, Ordering::SeqCst));
        PassRecorder::new(id, self.weak.upgrade().unwrap(), pipeline)
    }
}