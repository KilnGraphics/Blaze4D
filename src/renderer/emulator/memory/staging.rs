use std::sync::Arc;
use ash::vk;
use crate::prelude::DeviceContext;
use crate::util::alloc::RingAllocator;
use crate::vk::objects::allocator::{Allocation, AllocationStrategy};

pub struct StagingAllocationId {
    buffer_id: u16,
    slot_id: u16,
}

pub struct StagingMemoryPool {
    device: Arc<DeviceContext>,
    next_buffer_id: u16,
    current_buffer_id: u16,
    current_buffer: StagingBuffer,
    old_buffers: Vec<(u16, StagingBuffer)>,

    /// Multiplier applied to the size of a new backing buffer allocation.
    /// `0` is a multiplier of 1.0 and [`u8::MAX`] a multiplier of 2.0
    over_allocation: u8,

    /// The threshold of used memory at which point a backing buffer is reduced in size.
    /// `0` defines a threshold of `0%` i.e. never reduce and [`u8::MAX`] a threshold of `100%` i.e.
    /// always reduce.
    reduce_threshold: u8,
}

impl StagingMemoryPool {
    const MIN_BUFFER_SIZE: vk::DeviceSize = 2u64.pow(24); // 16MB

    pub fn allocate(&mut self, size: vk::DeviceSize, alignment: vk::DeviceSize) -> (vk::Buffer, vk::DeviceSize, StagingAllocationId) {
        if let Some((buffer, offset, slot_id)) = self.current_buffer.try_allocate(size, alignment) {
            (buffer, offset, StagingAllocationId{ buffer_id: self.current_buffer_id, slot_id })
        } else {
            self.create_new_buffer(size);
            let (buffer, offset, slot_id) = self.current_buffer.try_allocate(size, alignment).unwrap();
            (buffer, offset, StagingAllocationId{ buffer_id: 0, slot_id })
        }
    }

    pub fn free(&mut self, allocation: StagingAllocationId) {
        if allocation.buffer_id == self.current_buffer_id {
            self.current_buffer.free(allocation.slot_id);
        } else {
            let mut delete = None;
            for (index, (id, buffer)) in self.old_buffers.iter_mut().enumerate() {
                if *id == allocation.buffer_id {
                    buffer.free(allocation.slot_id);
                    if buffer.is_empty() {
                        delete = Some(index);
                    }
                    break;
                }
            }
            if let Some(index) = delete {
                self.old_buffers.swap_remove(index);
            }
        }
    }

    fn create_new_buffer(&mut self, additional_size: vk::DeviceSize) {
        let mut usage_sum = self.current_buffer.used_byte_count();
        for (_, old) in &self.old_buffers {
            usage_sum += old.used_byte_count();
        }
        usage_sum += additional_size;

        let new_size = (usage_sum * (u8::MAX as u64)) / (self.over_allocation as u64);
        let new_size = std::cmp::max(new_size, Self::MIN_BUFFER_SIZE);

        // Yes this is slow but it shouldn't matter since we never have many buffers
        while self.is_id_unused(self.next_buffer_id) {
            // Technically there is a potential infinite loop here but at that point we would have
            // allocated at least 1TB of memory so i will accept this risk
            self.next_buffer_id = self.next_buffer_id.wrapping_add(1);
        }
        let id = self.next_buffer_id;
        self.next_buffer_id = self.next_buffer_id.wrapping_add(1);

        let buffer = StagingBuffer::new(self.device.clone(), new_size);

        let old = std::mem::replace(&mut self.current_buffer, buffer);
        self.old_buffers.push((self.current_buffer_id, old));
        self.current_buffer_id = id;
    }

    fn is_id_unused(&self, id: u16) -> bool {
        if id == self.current_buffer_id {
            return false;
        }
        for (old, _) in &self.old_buffers {
            if *old == id {
                return false;
            }
        }
        true
    }
}

struct StagingBuffer {
    device: Arc<DeviceContext>,
    buffer: vk::Buffer,
    allocation: Option<Allocation>,
    allocator: RingAllocator,
}

impl StagingBuffer {
    fn new(device: Arc<DeviceContext>, size: vk::DeviceSize) -> Self {
        let info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(vk::BufferUsageFlags::TRANSFER_SRC | vk::BufferUsageFlags::TRANSFER_DST)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe {
            device.vk().create_buffer(&info, None)
        }.unwrap_or_else(|err| {
            log::error!("Failed to create staging buffer {:?}", err);
            panic!();
        });

        let allocation = device.get_allocator().allocate_buffer_memory(buffer, &AllocationStrategy::AutoGpuCpu).unwrap_or_else(|err| {
            log::error!("Failed to allocate staging buffer memory {:?}", err);
            unsafe { device.vk().destroy_buffer(buffer, None); };
            panic!();
        });

        if let Err(err) = unsafe {
            device.vk().bind_buffer_memory(buffer, allocation.memory(), allocation.offset())
        } {
            log::error!("Failed to bind staging buffer memory {:?}", err);
            unsafe { device.vk().destroy_buffer(buffer, None); };
            device.get_allocator().free(allocation);
            panic!();
        }

        Self {
            device,
            buffer,
            allocation: Some(allocation),
            allocator: RingAllocator::new(size)
        }
    }

    fn try_allocate(&mut self, size: vk::DeviceSize, alignment: vk::DeviceSize) -> Option<(vk::Buffer, vk::DeviceSize, u16)> {
        self.allocator.allocate(size, alignment).map(|(offset, slot)| {
            (self.buffer, offset, slot)
        })
    }

    fn free(&mut self, slot_id: u16) {
        self.allocator.free(slot_id);
    }

    fn is_empty(&self) -> bool {
        self.allocator.is_empty()
    }

    fn used_byte_count(&self) -> vk::DeviceSize {
        self.allocator.used_byte_count()
    }
}

impl Drop for StagingBuffer {
    fn drop(&mut self) {
        if !self.allocator.is_empty() {
            log::warn!("Destroying staging buffer with life allocations!");
        }
        unsafe {
            self.device.vk().destroy_buffer(self.buffer, None)
        };
        self.device.get_allocator().free(self.allocation.take().unwrap())
    }
}