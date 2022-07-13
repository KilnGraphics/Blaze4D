use std::ptr::NonNull;
use std::sync::Arc;

use ash::vk;

use crate::allocator::{Allocation, Allocator, HostAccess};
use crate::vk::objects::buffer::Buffer;

use crate::prelude::*;

pub(super) struct PoolAllocator {
    pools: [Pool; 8],
}

impl PoolAllocator {
    pub(super) fn new(device: Arc<DeviceFunctions>, allocator: Arc<Allocator>) -> Self {
        let pools = [
            Pool::new(device.clone(), allocator.clone(), 2u64.pow(10), 2048),
            Pool::new(device.clone(), allocator.clone(), 2u64.pow(21), 32),
            Pool::new(device.clone(), allocator.clone(), 2u64.pow(22), 16),
            Pool::new(device.clone(), allocator.clone(), 2u64.pow(23), 8),
            Pool::new(device.clone(), allocator.clone(), 2u64.pow(24), 8),
            Pool::new(device.clone(), allocator.clone(), 2u64.pow(25), 4),
            Pool::new(device.clone(), allocator.clone(), 2u64.pow(26), 4),
            Pool::new(device.clone(), allocator.clone(), 2u64.pow(27), 2),
        ];

        Self {
            pools
        }
    }

    pub(super) fn allocate(&mut self, min_size: vk::DeviceSize) -> PoolAllocation {
        for (index, pool) in self.pools.iter_mut().enumerate() {
            if pool.get_alloc_size() >= min_size {
                let (alloc, page_id) = pool.allocate();
                return PoolAllocation {
                    id: PoolAllocationId::new(index as u8, page_id, alloc.id),
                    buffer: alloc.buffer,
                    offset: alloc.offset,
                    size: alloc.size,
                    mapped: alloc.memory
                }
            }
        }
        // TODO support larger sizes
        panic!("Unsupported size. Larger sizes is a todo");
    }

    pub(super) fn free(&mut self, id: PoolAllocationId) {
        self.pools[id.get_pool_id() as usize].free(id.get_page_id(), id.get_slot_id());
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub(super) struct PoolAllocationId(u8, u8, u16);

impl PoolAllocationId {
    fn new(pool_id: u8, page_id: u8, slot_id: u16) -> Self {
        Self(pool_id, page_id, slot_id)
    }

    fn get_pool_id(&self) -> u8 {
        self.0
    }

    fn get_page_id(&self) -> u8 {
        self.1
    }

    fn get_slot_id(&self) -> u16 {
        self.2
    }
}

impl From<PoolAllocation> for PoolAllocationId {
    fn from(alloc: PoolAllocation) -> Self {
        alloc.id
    }
}

pub(super) struct PoolAllocation {
    id: PoolAllocationId,
    buffer: Buffer,
    offset: vk::DeviceSize,
    size: vk::DeviceSize,
    mapped: NonNull<u8>,
}

impl PoolAllocation {
    pub(super) fn get_id(&self) -> PoolAllocationId {
        self.id
    }

    pub(super) fn get_buffer(&self) -> Buffer {
        self.buffer
    }

    /// The offset into the buffer where this allocation range starts.
    ///
    /// This offset only applies to device side operations. It **must not** be applied to the pointer
    /// retrieved from [`PoolAllocation::get_memory`].
    pub(super) fn get_offset(&self) -> vk::DeviceSize {
        self.offset
    }

    /// The size in bytes of the allocation range.
    pub(super) fn get_size(&self) -> vk::DeviceSize {
        self.size
    }

    /// A pointer to the host mapped memory range for this allocation.
    pub(super) fn get_memory(&self) -> NonNull<u8> {
        self.mapped
    }
}

// Needed because of the NonNull<u8>
unsafe impl Send for PoolAllocation {
}

/// A pool providing allocations of a specific size.
struct Pool {
    device: Arc<DeviceFunctions>,
    allocator: Arc<Allocator>,
    pages: Vec<(u8, PoolPage)>,
    current_page: usize,
    alloc_size: vk::DeviceSize,
    slots_per_page: u16,
}

impl Pool {
    fn new(device: Arc<DeviceFunctions>, allocator: Arc<Allocator>, alloc_size: vk::DeviceSize, slots_per_page: u16) -> Self {
        let page = PoolPage::new(&device, &allocator, alloc_size, slots_per_page);

        Pool {
            device,
            allocator,
            pages: vec![(0, page)],
            current_page: 0,
            alloc_size,
            slots_per_page,
        }
    }

    fn get_alloc_size(&self) -> vk::DeviceSize {
        self.alloc_size
    }

    fn allocate(&mut self) -> (SlotAllocation, u8) {
        if self.pages[self.current_page].1.is_full() {
            self.current_page = self.find_or_create_page();
        }

        let (id, page) = &mut self.pages[self.current_page];
        (page.allocate().unwrap(), *id)
    }

    fn free(&mut self, page_id: u8, slot_id: u16) {
        // This isn't exceptionally fast but number of pages should be low and freeing is done in the worker thread so it shouldn't be a huge issue.
        for (id, page) in &mut self.pages {
            if *id == page_id {
                page.free(slot_id);
                return;
            }
        }
        panic!("Invalid page id in free");
    }

    /// Finds a non empty page or if none could be found creates a new page and returns its index.
    fn find_or_create_page(&mut self) -> usize {
        let mut max_id = 0;
        for (index, (id, page)) in self.pages.iter().enumerate() {
            if !page.is_full() {
                return index;
            }
            max_id = std::cmp::max(max_id, *id);
        }

        self.pages.push((max_id + 1, PoolPage::new(&self.device, &self.allocator, self.alloc_size, self.slots_per_page)));

        self.pages.len() - 1
    }
}

impl Drop for Pool {
    fn drop(&mut self) {
        let pages = std::mem::replace(&mut self.pages, Vec::new());
        for (_, page) in pages {
            page.destroy(&self.device, &self.allocator);
        }
    }
}

/// A single page of memory used by a pool.
///
/// A page represents one buffer which is divided into slots. Each slot can be individually allocated
/// and freed from the page.
struct PoolPage {
    buffer: Buffer,
    buffer_memory: NonNull<u8>,
    allocation: Allocation,
    slot_size: vk::DeviceSize,

    /// The list of all slots. If a slot is free it forms a linked list of free slots. If a slot is
    /// allocated its next index is undefined and must not be used.
    slots: Box<[PoolSlot]>,

    /// The number of free slots left.
    free_slots: usize,

    /// The next free slot index.
    next_slot: Option<u16>,
}

impl PoolPage {
    fn new(_: &DeviceFunctions, allocator: &Allocator, slot_size: vk::DeviceSize, slot_count: u16) -> Self {
        let byte_size = slot_size * (slot_count as vk::DeviceSize);
        let info = vk::BufferCreateInfo::builder()
            .size(byte_size)
            .usage(vk::BufferUsageFlags::TRANSFER_SRC | vk::BufferUsageFlags::TRANSFER_DST)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let (buffer, allocation, mapped) = unsafe {
            allocator.create_buffer(&info, HostAccess::Random, &format_args!("PoolPage"))
        }.unwrap();

        let buffer_memory = mapped.unwrap();

        let mut slots: Box<[_]> = (0..slot_count).map(|index|
            PoolSlot::new(Some(index + 1))
        ).collect();
        slots.last_mut().unwrap().set_next(None);

        Self {
            buffer: Buffer::new(buffer),
            buffer_memory,
            allocation,
            slot_size,
            slots,
            next_slot: Some(0),
            free_slots: slot_count as usize,
        }
    }

    fn destroy(self, _: &DeviceFunctions, allocator: &Allocator) {
        unsafe {
            allocator.destroy_buffer(self.buffer.get_handle(), self.allocation)
        }
    }

    /// Attempts to allocate one slot from the page.
    ///
    /// Returns the buffer, a pointer to the mapped memory already offset for the slot and the slot
    /// id.
    fn allocate(&mut self) -> Option<SlotAllocation> {
        let slot = self.allocate_slot()?;
        let memory = self.get_slot_memory(slot);

        Some(SlotAllocation {
            id: slot,
            buffer: self.buffer,
            offset: (slot as vk::DeviceSize) * self.slot_size,
            size: self.slot_size,
            memory
        })
    }

    /// Frees some previously allocated slot
    ///
    /// # Safety
    /// The id must have previously been allocated by a call to [`PoolPage::allocate`] from this
    /// page and it must not have been freed since then.
    fn free(&mut self, id: u16) {
        self.free_slot(id);
    }

    /// Returns true if there are no free slots in this page.
    fn is_full(&self) -> bool {
        self.free_slots == 0
    }

    fn get_slot_memory(&self, slot: u16) -> NonNull<u8> {
        let offset = (self.slot_size as isize) * (slot as isize);
        unsafe {
            NonNull::new_unchecked(self.buffer_memory.as_ptr().offset(offset))
        }
    }

    fn allocate_slot(&mut self) -> Option<u16> {
        if let Some(next) = self.next_slot.take() {
            self.next_slot = self.slots[next as usize].get_next();
            self.free_slots -= 1;

            Some(next)
        } else {
            None
        }
    }

    fn free_slot(&mut self, slot: u16) {
        self.slots[slot as usize].set_next(self.next_slot);
        self.next_slot = Some(slot);
        self.free_slots += 1;
    }
}

// Needed because of the NonNull<u8>
unsafe impl Send for PoolPage {
}

/// Represents one slot of the memory page. The slots form a linked list via indices pointing to the
/// next free slot.
struct PoolSlot {
    /// The next free slot. If this is [`u16::MAX`] there is no next slot.
    next: u16,
}

impl PoolSlot {
    fn new(next: Option<u16>) -> Self {
        let next = next.unwrap_or(u16::MAX);

        Self {
            next,
        }
    }

    fn get_next(&self) -> Option<u16> {
        if self.next != u16::MAX {
            Some(self.next)
        } else {
            None
        }
    }

    fn set_next(&mut self, next: Option<u16>) {
        self.next = next.unwrap_or(u16::MAX);
    }
}

struct SlotAllocation {
    id: u16,
    buffer: Buffer,
    offset: vk::DeviceSize,
    size: vk::DeviceSize,
    memory: NonNull<u8>,
}