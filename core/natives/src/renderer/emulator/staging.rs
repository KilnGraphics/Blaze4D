use std::ptr::NonNull;
use std::sync::Arc;

use ash::vk;
use crate::allocator::{Allocation, HostAccess};

use crate::prelude::DeviceContext;
use crate::util::alloc::RingAllocator;

pub(super) struct StagingMemory2 {
    device: Arc<DeviceContext>,
    main_pools: Vec<Pool2>,
    main_index: usize,
}

impl StagingMemory2 {
    const MAIN_POOL_SIZE: u64 = 16 * 1024 * 1024;
    const POOL_DESTROY_TIME: std::time::Duration = std::time::Duration::from_secs(10);

    pub(super) fn new(device: Arc<DeviceContext>) -> Self {
        Self {
            device,
            main_pools: Vec::new(),
            main_index: 0,
        }
    }

    /// Allocates a block of `size` bytes with `alignment` for use as staging memory.
    ///
    /// The allocation must later be freed with a call to [`StagingMemory2::free`] with the returned
    /// [`AllocationId2`].
    ///
    /// The buffer and memory returned must not be used after this struct is destroyed or a
    /// corresponding call to [`StagingMemory2::free`] has been made.
    pub(super) fn allocate(&mut self, size: u64, alignment: u64) -> (StagingAllocation2, StagingAllocationId2) {
        if size <= Self::MAIN_POOL_SIZE {
            self.allocate_main(size, alignment)
        } else {
            todo!()
        }
    }

    pub(super) unsafe fn free(&mut self, mut allocation: StagingAllocationId2) {
        match allocation.consume() {
            AllocationInfo::Main(index) => {
                let pool = &mut self.main_pools[index];
                pool.free();
                if pool.is_empty() && (index < self.main_index) {
                    // We want to prioritize the lower index pools so that we can potentially free pools that are not needed.
                    self.main_index = index;
                }
            }
            AllocationInfo::None => panic!("None allocation was passed to free"),
        }
    }

    pub(super) fn update(&mut self) {
        let now = std::time::Instant::now();
        while let Some(pool) = self.main_pools.last() {
            // We can only destroy the last pool if it is empty as otherwise the indices in AllocationId's would get messed up
            if pool.is_empty() && (now.duration_since(pool.get_timestamp()) >= Self::POOL_DESTROY_TIME) {
                self.main_pools.pop();
            } else {
                break;
            }
        }
    }

    fn allocate_main(&mut self, size: u64, alignment: u64) -> (StagingAllocation2, StagingAllocationId2) {
        if self.main_pools.is_empty() {
            self.main_pools.push(Pool2::new(self.device.clone(), Self::MAIN_POOL_SIZE).expect("Failed to create main pool"));
            self.main_index = 0;
        }

        let mut index = self.main_index;
        loop {
            let pool = &mut self.main_pools[index];
            if let Some((mapped_memory, buffer, offset)) = pool.allocate(size, alignment) {
                self.main_index = index;
                let alloc = StagingAllocation2 {
                    size,
                    buffer,
                    buffer_offset: offset,
                    mapped_memory,
                };
                let id = StagingAllocationId2(AllocationInfo::Main(index));

                return (alloc, id);
            }

            index = (index + 1) % self.main_pools.len();
            if index == self.main_index {
                // We have attempted all current pools. Need to create a new one.
                self.main_pools.push(Pool2::new(self.device.clone(), Self::MAIN_POOL_SIZE).expect("Failed to create main pool"));
                index = self.main_pools.len() - 1;
            }
        }
    }
}

pub(super) struct StagingAllocation2 {
    pub size: u64,
    pub buffer: vk::Buffer,
    pub buffer_offset: u64,
    pub mapped_memory: NonNull<u8>,
}

pub(super) struct StagingAllocationId2(AllocationInfo);

impl StagingAllocationId2 {
    fn consume(&mut self) -> AllocationInfo {
        std::mem::replace(&mut self.0, AllocationInfo::None)
    }
}

impl Drop for StagingAllocationId2 {
    fn drop(&mut self) {
        match &self.0 {
            AllocationInfo::None => {},
            _ => log::warn!("Emulator staging AllocationId has been dropped. Potential memory leak!"),
        }
    }
}

enum AllocationInfo {
    Main(usize),
    None,
}

struct Pool2 {
    device: Arc<DeviceContext>,
    buffer: vk::Buffer,
    allocation: Allocation,
    mapped: NonNull<u8>,
    size: u64,
    current_offset: u64,
    active_alloc_count: u64,
    timestamp: std::time::Instant,
}

impl Pool2 {
    fn new(device: Arc<DeviceContext>, size: u64) -> Option<Self> {
        let info = vk::BufferCreateInfo::builder()
            .usage(vk::BufferUsageFlags::TRANSFER_SRC | vk::BufferUsageFlags::TRANSFER_DST)
            .size(size)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let (buffer, allocation, mapped) = unsafe {
            device.get_allocator().create_buffer(&info, HostAccess::Random, &format_args!("EmulatorStagingBuffer"))
        }?;

        Some(Self {
            device,
            buffer,
            allocation,
            mapped: mapped.unwrap(),
            size,
            current_offset: 0,
            active_alloc_count: 0,
            timestamp: std::time::Instant::now(),
        })
    }

    /// Attempts to allocate some block of staging memory. If allocation fails [`None`] is returned.
    ///
    /// Any call to this function must later be matched by a call to [`Pool2::free`]. Failing to do
    /// so will eventually lead to the pool failing all allocations even if space would be
    /// available until all allocations have been freed.
    ///
    /// The buffer and memory returned must not be used after this struct is destroyed or a
    /// corresponding call to [`Pool2::free`] has been made.
    fn allocate(&mut self, size: u64, alignment: u64) -> Option<(NonNull<u8>, vk::Buffer, u64)> {
        let alignment = if alignment == 0 {
            1
        } else {
            alignment
        };

        let aligned_base = crate::util::alloc::next_aligned(self.current_offset, alignment);
        if aligned_base + size > self.size {
            return None;
        }

        let ptr = unsafe {
            // TODO this cast is not generally safe how do we improve this?
            NonNull::new(self.mapped.as_ptr().offset(aligned_base as isize))
        }.unwrap();

        self.current_offset = aligned_base + size;
        self.active_alloc_count += 1;
        self.timestamp = std::time::Instant::now();

        Some((ptr, self.buffer, aligned_base))
    }

    /// Frees a previously made allocation
    ///
    /// # Safety
    /// Let a be the number of successful allocations on this struct and f be the number of calls to
    /// this function. Then before calling this function a - f must be greater than 0.
    ///
    /// Any objects and memory previously allocated by a call to [`Pool2::allocate`] corresponding
    /// to this call to free must not be used anymore. This includes that any submissions must have
    /// finished execution.
    unsafe fn free(&mut self) {
        self.active_alloc_count -= 1;
        if self.active_alloc_count == 0 {
            self.current_offset = 0;
        }
    }

    /// Returns true if there are no active allocations.
    fn is_empty(&self) -> bool {
        self.active_alloc_count == 0
    }

    /// Returns the time of the last successful allocation from this pool. If no allocation has been
    /// made yet returns the time when this pool was created.
    fn get_timestamp(&self) -> std::time::Instant {
        self.timestamp
    }
}

impl Drop for Pool2 {
    fn drop(&mut self) {
        if self.active_alloc_count != 0 {
            log::warn!("Called Pool::drop while active allocations is not 0");
        }
        unsafe { self.device.get_allocator().destroy_buffer(self.buffer, self.allocation) };
    }
}

unsafe impl Send for Pool2 {
}









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

    pub(super) fn new(device: Arc<DeviceContext>) -> Self {
        let current_buffer = StagingBuffer::new(device.clone(), Self::MIN_BUFFER_SIZE);

        Self {
            device,
            next_buffer_id: 1,
            current_buffer_id: 0,
            current_buffer,
            old_buffers: Vec::new(),
            over_allocation: 76,
            reduce_threshold: 127
        }
    }

    pub(super) fn allocate(&mut self, size: vk::DeviceSize, alignment: vk::DeviceSize) -> (StagingAllocation, StagingAllocationId) {
        if let Some((alloc, slot_id)) = self.current_buffer.try_allocate(size, alignment) {
            (alloc, StagingAllocationId{ buffer_id: self.current_buffer_id, slot_id })
        } else {
            self.create_new_buffer(size);
            let (alloc, slot_id) = self.current_buffer.try_allocate(size, alignment).unwrap();
            (alloc, StagingAllocationId{ buffer_id: 0, slot_id })
        }
    }

    pub(super) fn free(&mut self, allocation: StagingAllocationId) {
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

        let new_size = usage_sum + ((usage_sum * (self.over_allocation as u64)) / (u8::MAX as u64));
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
    mapped_ptr: NonNull<u8>,
    allocation: Allocation,
    allocator: RingAllocator,
}

impl StagingBuffer {
    fn new(device: Arc<DeviceContext>, size: vk::DeviceSize) -> Self {
        let info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(vk::BufferUsageFlags::TRANSFER_SRC | vk::BufferUsageFlags::TRANSFER_DST)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let (buffer, allocation, mapped_ptr) = unsafe {
            device.get_allocator().create_buffer(&info, HostAccess::Random, &format_args!("StagingBuffer"))
        }.unwrap();

        Self {
            device,
            buffer,
            mapped_ptr: mapped_ptr.unwrap(),
            allocation,
            allocator: RingAllocator::new(size)
        }
    }

    fn try_allocate(&mut self, size: vk::DeviceSize, alignment: vk::DeviceSize) -> Option<(StagingAllocation, u16)> {
        self.allocator.allocate(size, alignment).map(|(offset, slot)| {
            let alloc = StagingAllocation {
                buffer: self.buffer,
                offset,
                mapped: unsafe { NonNull::new_unchecked(self.mapped_ptr.as_ptr().offset(offset as isize)) }
            };
            (alloc, slot)
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
            self.device.get_allocator().destroy_buffer(self.buffer, self.allocation)
        };
    }
}

unsafe impl Send for StagingBuffer { // Needed because of NonNull<u8>
}
unsafe impl Sync for StagingBuffer { // Needed because of NonNull<u8>
}

pub(super) struct StagingAllocation {
    pub(super) buffer: vk::Buffer,
    pub(super) offset: vk::DeviceSize,
    pub(super) mapped: NonNull<u8>,
}

unsafe impl Send for StagingAllocation { // Needed because of NonNull<u8>
}
unsafe impl Sync for StagingAllocation { // Needed because of NonNull<u8>
}