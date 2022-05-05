use std::collections::VecDeque;
use ash::vk;
use crate::vk::objects::allocator::Allocation;
use crate::vk::objects::buffer::Buffer;
use crate::vk::objects::semaphore::SemaphoreOp;

pub struct BufferPool {
    available_buffers: VecDeque<Buffer>,

}

impl BufferPool {
    pub fn new() -> Self {
        todo!()
    }

    /// (buffer, size, last_queue, wait_op)
    /// If the last queue is different from the transfer queue it is guaranteed that a release
    /// operation to the transfer queue has already been submitted.
    pub fn allocate_buffer(&mut self, min_size: usize) -> (Buffer, usize, u32, Option<SemaphoreOp>) {
        todo!()
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