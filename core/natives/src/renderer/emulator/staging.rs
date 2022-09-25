use std::ptr::NonNull;
use std::sync::Arc;

use ash::vk;
use crate::allocator::{Allocation, HostAccess};

use crate::prelude::DeviceContext;

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