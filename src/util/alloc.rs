//! Utilities to support creating device memory allocators

use ash::vk;

pub struct RingAllocator {
    size: vk::DeviceSize,
    head: vk::DeviceSize,
    tail: vk::DeviceSize,
    alloc_list_head: Option<u16>,
    alloc_list_tail: Option<u16>,
    free_list: Option<u16>,
    slots: Vec<RingAllocatorSlot>,
}

impl RingAllocator {
    pub fn new(size: vk::DeviceSize) -> Self {
        let slots = vec![RingAllocatorSlot::new(Some(1)), RingAllocatorSlot::new(None)];

        Self {
            size,
            head: 0,
            tail: 0,
            alloc_list_head: None,
            alloc_list_tail: None,
            free_list: Some(0),
            slots,
        }
    }

    pub fn allocate(&mut self, size: u64, alignment: u64) -> Option<(vk::DeviceSize, u16)> {
        let next = Self::next_aligned(self.head, alignment);

        let base;
        let end;
        if next + size > self.size {
            if size > self.tail {
                return None; // Out of memory
            }

            base = 0;
            end = size;
        } else {
            base = next;
            end = next + size;
        }

        if let Some(slot) = self.push_slot(end) {
            // Allocation successful. We only update state now to deal with potential errors
            self.head = end;

            Some((base, slot))
        } else {
            None
        }
    }

    pub fn free(&mut self, id: u16) {
        self.slots[id as usize].set_free(true);

        if self.alloc_list_tail == Some(id) {
            // This is the oldest slot. We now have to free it and any other connected free slots in the chain.
            let mut max_end = self.tail;
            let mut current = id as usize;
            while self.slots[current].is_free() {
                let slot = &mut self.slots[current];
                max_end = slot.get_end_offset();

                let next_slot = slot.get_next_slot();
                self.alloc_list_tail = next_slot;

                // Insert the slot into the free list
                slot.set_next_slot(self.free_list);
                self.free_list = Some(id as u16);

                if let Some(next_slot) = next_slot {
                    current = next_slot as usize;
                } else {
                    // The alloc list is now empty
                    self.alloc_list_head = None;
                    break;
                }
            }

            self.tail = max_end;
        }
    }

    fn push_slot(&mut self, end_offset: vk::DeviceSize) -> Option<u16> {
        let next_slot= self.free_list.or_else(|| {
            self.expand_slots(16);
            self.free_list
        })?;

        if let Some(head) = self.alloc_list_head {
            self.slots[head as usize].set_next_slot(Some(next_slot));
        }

        let slot = &mut self.slots[next_slot as usize];
        self.free_list = slot.get_next_slot();

        slot.set_free(false);
        slot.set_end_offset(end_offset);
        slot.set_next_slot(None);

        self.alloc_list_head = Some(next_slot);
        Some(next_slot)
    }

    fn expand_slots(&mut self, mut new: u16) {
        if (self.slots.len() + new as usize) > RingAllocatorSlot::MAX_SLOT_INDEX {
            new = (RingAllocatorSlot::MAX_SLOT_INDEX - self.slots.len()) as u16;
        }
        if new == 0 {
            return;
        }

        self.slots.reserve(new as usize);
        let base = self.slots.len() as u16;
        for i in base..(base + new - 1) {
            self.slots.push(RingAllocatorSlot::new(Some(i + 1)))
        }
        self.slots.push(RingAllocatorSlot::new(None));

        if let Some(head) = self.free_list {
            self.slots[head as usize].set_next_slot(Some(base));
        } else {
            self.free_list = Some(base);
        }
    }

    fn next_aligned(base: vk::DeviceSize, alignment: vk::DeviceSize) -> vk::DeviceSize {
        let rem = base % alignment;
        if rem == 0 {
            base
        } else {
            let diff = alignment - rem;
            base + diff
        }
    }
}

struct RingAllocatorSlot {
    /// Packed data format:
    /// - `end_offset` (bits 0-46): The offset of the first byte after the memory regions.
    /// - `free` (bit 47): Set to true if the allocation has been freed.
    /// - `next_slot` (bits 48-63): The index of the next slot in the linked list. If all bits are
    /// set to 1 this slot is the end of the list.
    payload: u64,
}

impl RingAllocatorSlot {
    const MAX_END_OFFSET: u64 = Self::END_OFFSET_MASK;
    const MAX_SLOT_INDEX: usize = ((u16::MAX - 1) as usize);

    const END_OFFSET_MASK: u64 = (u64::MAX >> 17);

    const FREE_MASK: u64 = (1u64 << 47);

    const NEXT_SLOT_OFFSET: u8 = 48;
    const NEXT_SLOT_MASK: u64 = ((u16::MAX as u64) << 48);

    #[inline]
    fn new(next_slot: Option<u16>) -> Self {
        let mut result = Self {
            payload: Self::NEXT_SLOT_MASK,
        };
        result.set_next_slot(next_slot);
        result
    }

    #[inline]
    fn set_end_offset(&mut self, end_offset: vk::DeviceSize) {
        assert_eq!(end_offset & !Self::END_OFFSET_MASK, 0u64);
        self.payload = (self.payload & !Self::END_OFFSET_MASK) | end_offset;
    }

    #[inline]
    fn get_end_offset(&self) -> vk::DeviceSize {
        self.payload & Self::END_OFFSET_MASK
    }

    #[inline]
    fn set_free(&mut self, free: bool) {
        let mut tmp = self.payload & !Self::FREE_MASK;
        if free {
            tmp |= Self::FREE_MASK;
        }
        self.payload = tmp;
    }

    #[inline]
    fn is_free(&self) -> bool {
        (self.payload & Self::FREE_MASK) == Self::FREE_MASK
    }

    #[inline]
    fn set_next_slot(&mut self, next_slot: Option<u16>) {
        let mut tmp = self.payload & !Self::NEXT_SLOT_MASK;
        if let Some(next_slot) = next_slot {
            let next_slot = next_slot as usize;
            assert!(next_slot <= Self::MAX_SLOT_INDEX);
            tmp |= (next_slot as u64) << Self::NEXT_SLOT_OFFSET;
        } else {
            tmp |= Self::NEXT_SLOT_MASK;
        }
        self.payload = tmp;
    }

    #[inline]
    fn get_next_slot(&self) -> Option<u16> {
        let masked = self.payload & Self::NEXT_SLOT_MASK;
        if masked == Self::NEXT_SLOT_MASK {
            None
        } else {
            Some((masked >> Self::NEXT_SLOT_OFFSET) as u16)
        }
    }
}

// Make sure we didnt mess up the bitmasks
const_assert_eq!(RingAllocatorSlot::END_OFFSET_MASK & RingAllocatorSlot::FREE_MASK & RingAllocatorSlot::NEXT_SLOT_MASK, 0u64);
const_assert_eq!(RingAllocatorSlot::END_OFFSET_MASK | RingAllocatorSlot::FREE_MASK | RingAllocatorSlot::NEXT_SLOT_MASK, u64::MAX);
const_assert_eq!(RingAllocatorSlot::NEXT_SLOT_MASK >> RingAllocatorSlot::NEXT_SLOT_OFFSET, u16::MAX as u64);
const_assert_eq!(((RingAllocatorSlot::MAX_SLOT_INDEX as u64) << RingAllocatorSlot::NEXT_SLOT_OFFSET) & !RingAllocatorSlot::NEXT_SLOT_MASK, 0u64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_allocator_slot() {
        let mut slot = RingAllocatorSlot::new(None);
        assert_eq!(slot.get_next_slot(), None);

        slot.set_free(true);
        assert_eq!(slot.is_free(), true);
        assert_eq!(slot.get_next_slot(), None);

        slot.set_end_offset(0);
        assert_eq!(slot.get_end_offset(), 0);
        assert_eq!(slot.is_free(), true);
        assert_eq!(slot.get_next_slot(), None);

        slot.set_end_offset(2355);
        assert_eq!(slot.get_end_offset(), 2355);
        assert_eq!(slot.is_free(), true);
        assert_eq!(slot.get_next_slot(), None);

        slot.set_end_offset(RingAllocatorSlot::MAX_END_OFFSET);
        assert_eq!(slot.get_end_offset(), RingAllocatorSlot::MAX_END_OFFSET);
        assert_eq!(slot.is_free(), true);
        assert_eq!(slot.get_next_slot(), None);

        slot.set_free(false);
        assert_eq!(slot.is_free(), false);
        assert_eq!(slot.get_next_slot(), None);

        slot.set_end_offset(0);
        assert_eq!(slot.get_end_offset(), 0);
        assert_eq!(slot.is_free(), false);
        assert_eq!(slot.get_next_slot(), None);

        slot.set_end_offset(2355);
        assert_eq!(slot.get_end_offset(), 2355);
        assert_eq!(slot.is_free(), false);
        assert_eq!(slot.get_next_slot(), None);

        slot.set_end_offset(RingAllocatorSlot::MAX_END_OFFSET);
        assert_eq!(slot.get_end_offset(), RingAllocatorSlot::MAX_END_OFFSET);
        assert_eq!(slot.is_free(), false);
        assert_eq!(slot.get_next_slot(), None);

        let mut slot = RingAllocatorSlot::new(Some(0));
        assert_eq!(slot.get_next_slot(), Some(0));

        slot.set_free(true);
        assert_eq!(slot.is_free(), true);
        assert_eq!(slot.get_next_slot(), Some(0));

        slot.set_end_offset(0);
        assert_eq!(slot.get_end_offset(), 0);
        assert_eq!(slot.is_free(), true);
        assert_eq!(slot.get_next_slot(), Some(0));

        slot.set_end_offset(2355);
        assert_eq!(slot.get_end_offset(), 2355);
        assert_eq!(slot.is_free(), true);
        assert_eq!(slot.get_next_slot(), Some(0));

        slot.set_end_offset(RingAllocatorSlot::MAX_END_OFFSET);
        assert_eq!(slot.get_end_offset(), RingAllocatorSlot::MAX_END_OFFSET);
        assert_eq!(slot.is_free(), true);
        assert_eq!(slot.get_next_slot(), Some(0));

        slot.set_free(false);
        assert_eq!(slot.is_free(), false);
        assert_eq!(slot.get_next_slot(), Some(0));

        slot.set_end_offset(0);
        assert_eq!(slot.get_end_offset(), 0);
        assert_eq!(slot.is_free(), false);
        assert_eq!(slot.get_next_slot(), Some(0));

        slot.set_end_offset(2355);
        assert_eq!(slot.get_end_offset(), 2355);
        assert_eq!(slot.is_free(), false);
        assert_eq!(slot.get_next_slot(), Some(0));

        slot.set_end_offset(RingAllocatorSlot::MAX_END_OFFSET);
        assert_eq!(slot.get_end_offset(), RingAllocatorSlot::MAX_END_OFFSET);
        assert_eq!(slot.is_free(), false);
        assert_eq!(slot.get_next_slot(), Some(0));

        let mut slot = RingAllocatorSlot::new(Some(4652));
        assert_eq!(slot.get_next_slot(), Some(4652));

        slot.set_free(true);
        assert_eq!(slot.is_free(), true);
        assert_eq!(slot.get_next_slot(), Some(4652));

        slot.set_end_offset(0);
        assert_eq!(slot.get_end_offset(), 0);
        assert_eq!(slot.is_free(), true);
        assert_eq!(slot.get_next_slot(), Some(4652));

        slot.set_end_offset(2355);
        assert_eq!(slot.get_end_offset(), 2355);
        assert_eq!(slot.is_free(), true);
        assert_eq!(slot.get_next_slot(), Some(4652));

        slot.set_end_offset(RingAllocatorSlot::MAX_END_OFFSET);
        assert_eq!(slot.get_end_offset(), RingAllocatorSlot::MAX_END_OFFSET);
        assert_eq!(slot.is_free(), true);
        assert_eq!(slot.get_next_slot(), Some(4652));

        slot.set_free(false);
        assert_eq!(slot.is_free(), false);
        assert_eq!(slot.get_next_slot(), Some(4652));

        slot.set_end_offset(0);
        assert_eq!(slot.get_end_offset(), 0);
        assert_eq!(slot.is_free(), false);
        assert_eq!(slot.get_next_slot(), Some(4652));

        slot.set_end_offset(2355);
        assert_eq!(slot.get_end_offset(), 2355);
        assert_eq!(slot.is_free(), false);
        assert_eq!(slot.get_next_slot(), Some(4652));

        slot.set_end_offset(RingAllocatorSlot::MAX_END_OFFSET);
        assert_eq!(slot.get_end_offset(), RingAllocatorSlot::MAX_END_OFFSET);
        assert_eq!(slot.is_free(), false);
        assert_eq!(slot.get_next_slot(), Some(4652));

        let mut slot = RingAllocatorSlot::new(Some(RingAllocatorSlot::MAX_SLOT_INDEX as u16));
        assert_eq!(slot.get_next_slot(), Some(RingAllocatorSlot::MAX_SLOT_INDEX as u16));

        slot.set_free(true);
        assert_eq!(slot.is_free(), true);
        assert_eq!(slot.get_next_slot(), Some(RingAllocatorSlot::MAX_SLOT_INDEX as u16));

        slot.set_end_offset(0);
        assert_eq!(slot.get_end_offset(), 0);
        assert_eq!(slot.is_free(), true);
        assert_eq!(slot.get_next_slot(), Some(RingAllocatorSlot::MAX_SLOT_INDEX as u16));

        slot.set_end_offset(2355);
        assert_eq!(slot.get_end_offset(), 2355);
        assert_eq!(slot.is_free(), true);
        assert_eq!(slot.get_next_slot(), Some(RingAllocatorSlot::MAX_SLOT_INDEX as u16));

        slot.set_end_offset(RingAllocatorSlot::MAX_END_OFFSET);
        assert_eq!(slot.get_end_offset(), RingAllocatorSlot::MAX_END_OFFSET);
        assert_eq!(slot.is_free(), true);
        assert_eq!(slot.get_next_slot(), Some(RingAllocatorSlot::MAX_SLOT_INDEX as u16));

        slot.set_free(false);
        assert_eq!(slot.is_free(), false);
        assert_eq!(slot.get_next_slot(), Some(RingAllocatorSlot::MAX_SLOT_INDEX as u16));

        slot.set_end_offset(0);
        assert_eq!(slot.get_end_offset(), 0);
        assert_eq!(slot.is_free(), false);
        assert_eq!(slot.get_next_slot(), Some(RingAllocatorSlot::MAX_SLOT_INDEX as u16));

        slot.set_end_offset(2355);
        assert_eq!(slot.get_end_offset(), 2355);
        assert_eq!(slot.is_free(), false);
        assert_eq!(slot.get_next_slot(), Some(RingAllocatorSlot::MAX_SLOT_INDEX as u16));

        slot.set_end_offset(RingAllocatorSlot::MAX_END_OFFSET);
        assert_eq!(slot.get_end_offset(), RingAllocatorSlot::MAX_END_OFFSET);
        assert_eq!(slot.is_free(), false);
        assert_eq!(slot.get_next_slot(), Some(RingAllocatorSlot::MAX_SLOT_INDEX as u16));
    }
}