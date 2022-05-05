mod pipeline;
mod buffer;
mod frame;
mod render_worker;

use std::ops::DerefMut;
use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::atomic::{AtomicU64, Ordering};
use ash::vk;
use concurrent_queue::ConcurrentQueue;
use crate::renderer::emulator::buffer::{BufferAllocation, BufferPool, BufferSubAllocator};
use crate::renderer::emulator::frame::FrameManager;
use crate::renderer::emulator::pipeline::PipelineId;
use crate::renderer::emulator::render_worker::{DrawTask, Share};
use crate::renderer::swapchain_manager::SwapchainInstance;
use crate::transfer::{BufferAvailabilityOp, BufferTransferRanges, Transfer};
use crate::vk::objects::buffer::Buffer;
use crate::vk::objects::semaphore::SemaphoreOps;

struct EmulatorRendererShare {
    transfer: Transfer,
    worker: Arc<Share>,
    frame_manager: FrameManager,
    buffer_pool: Mutex<BufferPool>,
}

impl EmulatorRendererShare {
    fn new(transfer: Transfer) -> Self {
        Self {
            transfer,
            worker: Arc::new(Share::new()),
            frame_manager: FrameManager::new(),
            buffer_pool: Mutex::new(BufferPool::new()),
        }
    }
}

struct EmulatorRenderer(Arc<EmulatorRendererShare>);

impl EmulatorRenderer {
    fn register_pipeline(&self) -> PipelineId {
        todo!()
    }

    pub fn start_frame(&self) {
        todo!()
    }
}

/// These are all objects which are not expected to change frequently. Things like the swapchain
/// and swapchain dependant objects.
struct StableObjects {
    swapchain: SwapchainInstance,
}