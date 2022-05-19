use std::collections::VecDeque;
use std::ops::{Deref, Index};

use ash::vk;

use crate::renderer::emulator::pass::PassId;
use crate::device::transfer::{BufferAvailabilityOp};
use crate::vk::objects::allocator::{Allocation, AllocationStrategy};
use crate::vk::objects::buffer::{Buffer, BufferId};
use crate::vk::objects::semaphore::{SemaphoreOp, SemaphoreOps};

use crate::prelude::*;

/// Manges the lifetime and allocation of buffers for the emulator renderer.
pub struct BufferPool {
    buffers: Vec<PoolBuffer>,
    device: DeviceEnvironment,
}

impl BufferPool {
    pub fn new(device: DeviceEnvironment) -> Self {
        Self {
            buffers: Vec::with_capacity(16),
            device,
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
    pub fn return_buffer(&mut self, buffer_id: BufferId, wait_op: Option<SemaphoreOp>) {
        for buffer in &mut self.buffers {
            if buffer.is(buffer_id) {
                buffer.transition_available(wait_op);
                return;
            }
        }
        panic!("Unknown buffer");
    }

    fn find_or_create_buffer(&mut self, min_size: usize) -> &mut PoolBuffer {
        if min_size > 256000000 {
            panic!("Fuck this buffer with size {:?}", min_size)
        }

        let mut index = None;
        for (buffer_index, buffer) in self.buffers.iter().enumerate() {
            if buffer.available() {
                index = Some(buffer_index);
                break;
            }
        }

        if let Some(index) = index {
            self.buffers.get_mut(index).unwrap()

        } else {
            let buffer = PoolBuffer::new(256000000);
            self.buffers.push(buffer);

            self.buffers.last_mut().unwrap()
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
            state: BufferState::Available(None),
            marked: false,
        }
    }

    fn available(&self) -> bool {
        if self.marked {
            return false;
        }

        match &self.state {
            BufferState::Available(_) => true,
            _ => false
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
        if let BufferState::Available(op) = self.state {
            self.state = BufferState::Unavailable;
            return op;
        } else {
            panic!("Invalid state");
        }
    }

    fn transition_available(&mut self, op: Option<SemaphoreOp>) {
        if let BufferState::Unavailable = self.state {
            self.state = BufferState::Available(op);
        } else {
            panic!("Invalid state");
        }
    }

    fn try_destroy(&mut self, device: &DeviceEnvironment) -> bool {
        if let BufferState::Available(wait_op) = &self.state {
            if let Some(wait_op) = wait_op {
                if let Some(value) = wait_op.value {
                    let semaphore_value = unsafe {
                        device.vk().get_semaphore_counter_value(wait_op.semaphore)
                    }.unwrap();
                    if semaphore_value < value {
                        return false;
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
            true
        } else {
            false
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum BufferState {
    Available(Option<SemaphoreOp>),
    Unavailable,
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