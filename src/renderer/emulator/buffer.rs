use std::collections::VecDeque;
use std::ops::{Deref, Index};

use ash::vk;

use crate::renderer::emulator::frame::FrameId;
use crate::transfer::{BufferAvailabilityOp, Transfer};
use crate::vk::objects::allocator::{Allocation, AllocationStrategy};
use crate::vk::objects::buffer::{Buffer, BufferId};
use crate::vk::objects::semaphore::{SemaphoreOp, SemaphoreOps};

use crate::prelude::*;

/// Manges the lifetime and allocation of buffers for the emulator renderer.
pub struct BufferPool {
    buffers: Vec<PoolBuffer>,
    estimator: Estimator,

    device: DeviceEnvironment,
}

impl BufferPool {
    pub fn new(device: DeviceEnvironment) -> Self {
        Self {
            buffers: Vec::with_capacity(16),
            estimator: Estimator::new(),
            device,
        }
    }

    /// Runs any queued up tasks which should not be run synchronously due to high performance
    /// impact.
    pub fn update(&mut self) {
        let mut has_destroy = false;
        for buffer in &mut self.buffers {
            if buffer.marked {
                buffer.try_destroy(&self.device);
                has_destroy = true;
            } else {
                buffer.ensure_init(&self.device);
            }
        }

        self.buffers.retain(|buffer| !(buffer.buffer.is_none() && buffer.marked));
    }

    /// Must be called at the end of each frame with the amount of bytes used during the frame.
    /// This is internally used to estimate memory usage.
    pub fn end_frame(&mut self, usage: usize) {
        self.estimator.push_frame(usage);

        if let Some(estimate) = self.estimator.get_estimate() {
            let mut buffer_count = 0usize;
            for buffer in &mut self.buffers {
                if buffer.marked {
                    continue;
                }

                if buffer_count >= estimate.buffer_count {
                    buffer.marked = true;
                    continue;
                }

                if estimate.satisfies(buffer.size) {
                    buffer_count += 1;
                } else {
                    buffer.marked = true;
                }
            }

            if estimate.buffer_count > buffer_count {
                for _ in buffer_count..estimate.buffer_count {
                    self.buffers.push(PoolBuffer::new(estimate.buffer_size));
                }
            }
        }
    }

    /// Returns a buffer from the pool of available buffers.
    ///
    /// The return value is the buffer, the size of the buffer and potentially a timeline semaphore
    /// wait operation.
    pub fn get_buffer(&mut self, min_size: usize) -> (Buffer, usize, Option<SemaphoreOp>) {
        // TODO fix this clone
        let device = self.device.clone();

        let buffer = self.find_or_create_buffer(min_size);

        let raw_buffer = buffer.ensure_init(&device);
        let op = buffer.transition_unavailable();

        (raw_buffer, buffer.size, op)
    }

    /// Returns a buffer previously allocated from the pool to the pool.
    ///
    /// The sequence number is a number used to influence the order in which buffers are returned by
    /// [`get_buffer`]. A higher sequence number indicates that a buffer has been used more recently
    /// and as such may incur a longer wait time.
    pub fn return_buffer(&mut self, buffer_id: BufferId, wait_op: Option<SemaphoreOp>, sequence_number: u64) {
        for buffer in &mut self.buffers {
            if buffer.is(buffer_id) {
                buffer.transition_available(wait_op, sequence_number);
                return;
            }
        }
        panic!("Unknown buffer");
    }

    fn find_or_create_buffer(&mut self, min_size: usize) -> &mut PoolBuffer {
        let mut best = None;
        for (index, buffer) in self.buffers.iter().enumerate() {
            if let Some(sequence_number) = buffer.available(min_size) {
                if let Some((_, best_num)) = best {
                    if best_num > sequence_number {
                        best = Some((index, sequence_number));
                    }
                } else {
                    best = Some((index, sequence_number));
                }
            }
        }

        if let Some((index, _)) = best {
            &mut self.buffers[index]
        } else {
            self.estimator.mark_upset();

            // TODO make better estimation?
            let sum = self.buffers.iter().fold(0, |old, buffer| old + buffer.size);
            self.buffers.push(PoolBuffer::new(std::cmp::max(min_size * 8, sum)));

            let index = self.buffers.len() - 1;
            &mut self.buffers[index]
        }
    }
}

struct PoolBuffer {
    buffer: Option<(Buffer, Allocation)>,
    size: usize,
    state: BufferState,
    marked: bool,
}

impl PoolBuffer {
    fn new(size: usize) -> Self {
        Self {
            buffer: None,
            size,
            state: BufferState::Available(None, 0),
            marked: false,
        }
    }

    fn available(&self, min_size: usize) -> Option<u64> {
        if self.size < min_size {
            return None;
        }

        match &self.state {
            BufferState::Available(_, sequence_number) => Some(*sequence_number),
            _ => None
        }
    }

    fn is(&self, buffer_id: BufferId) -> bool {
        if let Some((buffer, _)) = &self.buffer {
            buffer.get_id() == buffer_id
        } else {
            false
        }
    }

    fn ensure_init(&mut self, device: &DeviceEnvironment) -> Buffer {
        if let Some((buffer, _)) = &self.buffer {
            *buffer
        } else {
            let info = vk::BufferCreateInfo::builder()
                .size(self.size as vk::DeviceSize)
                .usage(vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::INDEX_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let handle = unsafe {
                device.vk().create_buffer(&info, None)
            }.unwrap();

            let allocation = device.get_allocator().allocate_buffer_memory(handle, &AllocationStrategy::AutoGpuOnly).unwrap();

            unsafe {
                device.vk().bind_buffer_memory(handle, allocation.memory(), allocation.offset())
            }.unwrap();

            let buffer = Buffer::new(handle);

            self.buffer = Some((buffer, allocation));

            buffer
        }
    }

    fn transition_unavailable(&mut self) -> Option<SemaphoreOp> {
        if let BufferState::Available(op, _) = self.state {
            self.state = BufferState::Unavailable;
            return op;
        } else {
            panic!("Invalid state");
        }
    }

    fn transition_available(&mut self, op: Option<SemaphoreOp>, sequence_number: u64) {
        if let BufferState::Unavailable = self.state {
            self.state = BufferState::Available(op, sequence_number);
        } else {
            panic!("Invalid state");
        }
    }

    fn try_destroy(&mut self, device: &DeviceEnvironment) {
        if let BufferState::Available(wait_op, _) = &self.state {
            if let Some(wait_op) = wait_op {
                if let Some(value) = wait_op.value {
                    let semaphore_value = unsafe {
                        device.vk().get_semaphore_counter_value(wait_op.semaphore)
                    }.unwrap();
                    if semaphore_value < value {
                        return;
                    }
                } else {
                    panic!("Wait semaphore must be a timeline semaphore");
                }
            }

            if let Some((buffer, allocation)) = self.buffer.take() {
                unsafe {
                    device.vk().destroy_buffer(buffer.get_handle(), None)
                };

                device.get_allocator().free(allocation);
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum BufferState {
    Available(Option<SemaphoreOp>, u64),
    Unavailable,
}

/// Used to predict the required number and size of buffers for optimal
struct Estimator {
    frame_history: Box<[usize]>,
    current_index: usize,
    history_depth: usize,
    sum_total: usize,
    multiplier: usize,
}

impl Estimator {
    pub fn new() -> Self {
        Self {
            frame_history: std::iter::repeat(0usize).take(100).collect(),
            current_index: 0,
            history_depth: 0,
            sum_total: 0,
            multiplier: 130,
        }
    }

    pub fn push_frame(&mut self, usage: usize) {
        self.current_index += 1;
        if self.current_index == self.frame_history.len() {
            self.current_index = 0;
        }

        if self.history_depth == self.frame_history.len() {
            self.sum_total -= self.frame_history[self.current_index];
        } else {
            self.history_depth += 1;
        }

        self.frame_history[self.current_index] = usage;
        self.sum_total += usage;
    }

    /// Informs the estimator that the current estimated usage was wildly incorrect.
    pub fn mark_upset(&mut self) {
        // The current implementation just resets the history buffer
        self.history_depth = 0;
        self.sum_total = 0;
    }

    /// Updates the used estimation multiplier.
    ///
    /// The multiplier is a percentage value. For example 100 does not change the estimation while
    /// 200 doubles it.
    pub fn set_multiplier(&mut self, multiplier: usize) {
        self.multiplier = multiplier;
    }

    /// Returns the current estimated memory usage of a frame.
    ///
    /// If no estimation is possible [`None`] is returned. This may happen at any moment for any
    /// reason so the caller must be able to deal with a [`None`] return value.
    pub fn get_estimate(&self) -> Option<UsageEstimate> {
        if self.history_depth > 0 {
            let estimate = self.sum_total / self.history_depth;
            let multiplied = (estimate * self.multiplier) / 100usize;
            let per_buffer = std::cmp::max(multiplied / 6, 32000000);

            Some(UsageEstimate {
                buffer_count: 6,
                buffer_size: per_buffer,
                update_at: per_buffer / 10,
            })
        } else {
            None
        }
    }
}

struct UsageEstimate {
    buffer_count: usize,
    buffer_size: usize,
    update_at: usize,
}

impl UsageEstimate {
    fn satisfies(&self, buffer_size: usize) -> bool {
        let diff = {
            if buffer_size > self.buffer_size {
                buffer_size - self.buffer_size
            } else {
                self.buffer_size - buffer_size
            }
        };

        diff < self.update_at
    }
}

pub struct BufferSubAllocator {
    buffer: Buffer,
    offset: usize,
    size: usize,
}

impl BufferSubAllocator {
    pub fn new(buffer: Buffer, size: usize) -> Self {
        Self {
            buffer,
            offset: 0,
            size,
        }
    }

    pub fn get_buffer(&self) -> Buffer {
        self.buffer
    }

    pub fn allocate(&mut self, size: usize, alignment: u32) -> Option<BufferAllocation> {
        let base_offset = Self::next_aligned(self.offset, alignment);
        let next_offset = base_offset + size;

        if next_offset > self.size {
            return None;
        }

        self.offset = next_offset;

        Some(BufferAllocation {
            buffer: self.buffer,
            offset: base_offset,
        })
    }

    fn next_aligned(offset: usize, alignment: u32) -> usize {
        let alignment = alignment as usize;
        let diff = offset % alignment;
        if diff == 0 {
            offset
        } else {
            offset + (alignment - diff)
        }
    }
}

pub struct BufferAllocation {
    pub buffer: Buffer,
    pub offset: usize,
}