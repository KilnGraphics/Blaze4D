mod pipeline;
mod buffer;
mod frame;

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use ash::vk;
use concurrent_queue::ConcurrentQueue;
use crate::renderer::emulator::buffer::BufferPool;
use crate::renderer::swapchain_manager::SwapchainInstance;

struct EmulatorRendererShare {
    render_tasks: ConcurrentQueue<()>,
    completion_tasks: ConcurrentQueue<()>,
    frame_manager: FrameManager,
}

impl EmulatorRendererShare {
    fn new() -> Self {
        Self {
            render_tasks: ConcurrentQueue::unbounded(),
            completion_tasks: ConcurrentQueue::bounded(3),
            frame_manager: FrameManager::new(),
        }
    }
}

struct EmulatorRenderer(Arc<EmulatorRendererShare>);

impl EmulatorRenderer {
    fn register_pipeline(&self) {

    }

    pub fn start_frame(&self) {
        todo!()
    }
}

struct FrameManager {
    current_frame_id: AtomicU64,
    last_submitted_id: AtomicU64,
    last_completed_id: AtomicU64,
}

/// Tracks global frame state.
impl FrameManager {
    fn new() -> Self {
        Self {
            current_frame_id: AtomicU64::new(0u64),
            last_submitted_id: AtomicU64::new(0u64),
            last_completed_id: AtomicU64::new(0u64),
        }
    }

    /// Validates that there is no active frame and returns the id of the next frame.
    /// If there is a active frame this function returns None.
    fn start_frame(&self) -> Option<u64> {
        let last_submitted = self.last_submitted_id.load(Ordering::SeqCst);
        // No need to load the current id since the last id must be equal to the current one anyways
        let next_id = last_submitted + 1;
        self.current_frame_id.compare_exchange(last_submitted, next_id, Ordering::SeqCst, Ordering::SeqCst).ok()
    }

    /// Marks a frame as submitted
    fn mark_submitted(&self, id: u64) {
        self.last_submitted_id.store(id, Ordering::SeqCst);
    }

    /// Marks a frame as completed.
    /// This will also mark the frame as submitted. This is useful in case a frame is aborted
    /// before submission.
    fn mark_completed(&self, id: u64) {
        // If a frame is not submitted it must be the current frame
        let last = id - 1;
        self.current_frame_id.compare_exchange(last, id, Ordering::SeqCst, Ordering::SeqCst);

        self.last_completed_id.store(id, Ordering::SeqCst);
    }

    /// Checks if a frame is submitted
    fn is_submitted(&self, id: u64) -> bool {
        id <= self.last_submitted_id.load(Ordering::SeqCst)
    }

    /// Checks if a frame is completed
    fn is_completed(&self, id: u64) -> bool {
        id <= self.last_completed_id.load(Ordering::SeqCst)
    }
}

struct FrameShare {
    buffers: Mutex<BufferPool>,
    object_queue: ConcurrentQueue<RecordedObject>,
}

impl FrameShare {
    pub fn record_object(&self, object: &ObjectData) {
        let vertex_size = self.get_pipeline_vertex_size(object.pipeline);
        let (vertex_buffer, index_buffer) = {
            let mut guard = self.buffers.lock().unwrap();
            (
                guard.reserve_memory(object.vertex_data, vertex_size),
                guard.reserve_memory(object.index_data, 4u32),
            )
        };

        let recorded = RecordedObject {
            vertex_buffer: vertex_buffer.buffer,
            first_vertex: (vertex_buffer.offset / (vertex_size as usize)) as u32,
            index_buffer: index_buffer.buffer,
            first_index: (index_buffer.offset / 4usize) as u32,
            vertex_count: object.draw_count,
        };
        self.object_queue.push(recorded);
    }

    pub fn end_frame(&self) {
        self.object_queue.close();
    }

    fn get_pipeline_vertex_size(&self, pipeline: u32) -> u32 {
        todo!()
    }
}

struct Frame {
    share: Arc<FrameShare>,
}

struct RecordedObject {
    vertex_buffer: vk::Buffer,
    first_vertex: u32,
    index_buffer: vk::Buffer,
    first_index: u32,
    vertex_count: u32,
}

struct ObjectData<'a> {
    vertex_data: &'a [u8],
    index_data: &'a [u8],
    pipeline: u32,
    draw_count: u32,
}

/// These are all objects which are not expected to change frequently. Things like the swapchain
/// and swapchain dependant objects.
struct StableObjects {
    swapchain: SwapchainInstance,
}