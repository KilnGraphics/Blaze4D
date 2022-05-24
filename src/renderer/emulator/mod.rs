mod buffer;
mod worker;
pub mod pipeline;
pub mod debug_pipeline;
pub mod pass;

use std::ops::DerefMut;
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use ash::vk;

use concurrent_queue::ConcurrentQueue;

use crate::device::device_utils::BlitPass;
use crate::renderer::emulator::buffer::{BufferAllocation, BufferPool, BufferSubAllocator};
use crate::renderer::emulator::pass::{PassRecorder, PassId};
use crate::renderer::emulator::worker::{DrawTask, run_worker, Share};
use crate::device::transfer::{BufferAvailabilityOp, BufferTransferRanges, Transfer};
use crate::objects::id::ImageId;
use crate::objects::{ObjectSet, ObjectSetProvider};
use crate::vk::objects::buffer::Buffer;

use crate::prelude::*;
use crate::vk::DeviceEnvironment;
use crate::vk::objects::allocator::{Allocation, AllocationStrategy};

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
            run_worker(device, share);
        });

        renderer
    }

    pub fn start_frame(&self, pipeline: Arc<dyn EmulatorPipeline>) -> PassRecorder {
        let id = PassId::from_raw(self.next_frame_id.fetch_add(1, Ordering::SeqCst));
        PassRecorder::new(id, self.weak.upgrade().unwrap(), pipeline)
    }
}