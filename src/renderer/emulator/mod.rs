mod pipeline;
mod buffer;
pub mod pass;
mod worker;

use std::iter::repeat_with;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use ash::vk;

use concurrent_queue::ConcurrentQueue;

use crate::device::device_utils::BlitPass;
use crate::renderer::emulator::buffer::{BufferAllocation, BufferPool, BufferSubAllocator};
use crate::renderer::emulator::pass::{Pass, PassId};
use crate::renderer::emulator::worker::{DrawTask, run_worker, Share};
use crate::device::transfer::{BufferAvailabilityOp, BufferTransferRanges, Transfer};
use crate::objects::id::ImageId;
use crate::objects::{ObjectSet, ObjectSetProvider};
use crate::vk::objects::buffer::Buffer;
use crate::vk::objects::semaphore::SemaphoreOps;

use crate::prelude::*;
use crate::vk::DeviceEnvironment;
use crate::vk::objects::allocator::{Allocation, AllocationStrategy};

pub use pipeline::RenderPath;
pub use pipeline::RenderConfiguration;
pub use pipeline::OutputConfiguration;
pub use pipeline::TestVertex;

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

    pub fn crate_test_render_path(&self) -> Arc<RenderPath> {
        Arc::new(RenderPath::new(self.device.clone()))
    }

    pub fn create_output_configuration(
        &self,
        render_configuration: Arc<RenderConfiguration>,
        output_size: Vec2u32,
        dst_images: &[ImageId],
        dst_set: ObjectSet,
        dst_format: vk::Format,
        final_layout: vk::ImageLayout
    ) -> Arc<OutputConfiguration> {

        Arc::new(OutputConfiguration::new(
            render_configuration,
            output_size,
            dst_images,
            dst_set,
            dst_format,
            final_layout
        ))
    }

    pub fn start_frame(&self, configuration: Arc<RenderConfiguration>) -> Pass {
        let id = PassId::from_raw(self.next_frame_id.fetch_add(1, Ordering::SeqCst));
        Pass::new(id, self.weak.upgrade().unwrap(), configuration)
    }
}