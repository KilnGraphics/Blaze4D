package me.hydos.rosella.memory.allocators;

import org.junit.jupiter.api.Test;

import java.time.Duration;
import java.util.*;

import static org.junit.jupiter.api.Assertions.*;

public class TestUnbackedRingAllocator {

    @Test
    void allocateFreeSingle() {
        int[] allocationSizesSuccess = new int[]{ 16, 15, 1020, 1024, 471, 981, 1};
        for(int size : allocationSizesSuccess) {
            UnbackedRingAllocator allocator = new UnbackedRingAllocator(1024);
            assertTrue(allocator.isEmpty());
            assertFalse(allocator.isFull());

            long alloc = allocator.allocate(size);
            assertNotEquals(-1, alloc, "Allocation failed with " + size);

            assertFalse(allocator.isEmpty(), "Should not be empty when allocating " + size);
            if(size != 1024) {
                assertFalse(allocator.isFull(), "Should not be full when allocating " + size);
            } else {
                assertTrue(allocator.isFull(), "Should be full after allocating 1020");
            }

            assertTimeoutPreemptively(Duration.ofSeconds(10), () -> allocator.free(alloc), "Free should not take over 10s for " + size);

            assertTrue(allocator.isEmpty(), "Should be empty after free " + size);
            assertFalse(allocator.isFull(), "Should not be full after free " + size);
        }

        int[] allocationSizesFail = new int[]{ 1025, 38924, Integer.MAX_VALUE };
        for(int size : allocationSizesFail) {
            UnbackedRingAllocator allocator = new UnbackedRingAllocator(1024);
            assertTrue(allocator.isEmpty());
            assertFalse(allocator.isFull());

            assertDoesNotThrow(() -> {
                long alloc = allocator.allocate(size);
                assertEquals(-1, alloc, "Allocation should fail when allocating " + size);

                assertTrue(allocator.isEmpty(), "Should be empty after failed allocation " + size);
                assertFalse(allocator.isFull(), "Should not be full after failed allocation " + size);
            });
        }

        int[] allocationSizesThrow = new int[]{ 0, -3, -1020, -1024, -1025, -349094, Integer.MIN_VALUE };
        for(int size : allocationSizesThrow) {
            UnbackedRingAllocator allocator = new UnbackedRingAllocator(1024);
            assertTrue(allocator.isEmpty());
            assertFalse(allocator.isFull());

            assertThrows(IllegalArgumentException.class, () -> allocator.allocate(size));

            assertTrue(allocator.isEmpty(), "Should be empty after failed allocation " + size);
            assertFalse(allocator.isFull(), "Should not be full after failed allocation " + size);
        }
    }

    @Test
    void allocateFreeSeqOrdered() {
        UnbackedRingAllocator allocator = new UnbackedRingAllocator(1024);
        Deque<Long> allocations = new ArrayDeque<>();

        for(int n = 0; n < 4; n++) {
            long head = 0;
            for (int i = 0; i < 32; i++) {
                long alloc = allocator.allocate(i + 1);
                assertNotEquals(alloc, -1);

                if(alloc < head) {
                    fail("Allocation overlaps with previous allocation");
                }
                head = alloc + i + 1;

                assertFalse(allocator.isEmpty());
                assertFalse(allocator.isFull());

                allocations.addFirst(alloc);
            }
            for(int i = 0; i < 32; i++) {
                long alloc = allocations.pollLast();

                assertFalse(allocator.isFull());
                assertFalse(allocator.isEmpty());

                assertTimeoutPreemptively(Duration.ofSeconds(10), () -> allocator.free(alloc));
            }

            assertTrue(allocator.isEmpty());
            assertFalse(allocator.isFull());
        }
    }

    @Test
    void allocateFreeSeqRandom() {
        Random random = new Random(59212);
        UnbackedRingAllocator allocator = new UnbackedRingAllocator(1024);
        List<Long> allocations = new ArrayList<>();

        for(int n = 0; n < 4; n++) {
            long head = 0;
            for (int i = 0; i < 32; i++) {
                long alloc = allocator.allocate(i + 1);
                assertNotEquals(alloc, -1);

                if(alloc < head) {
                    fail("Allocation overlaps with previous allocation");
                }
                head = alloc + i + 1;

                assertFalse(allocator.isEmpty());
                assertFalse(allocator.isFull());

                allocations.add(alloc);
            }
            for(int i = 0; i < 32; i++) {
                long alloc = allocations.remove(random.nextInt(allocations.size()));

                assertFalse(allocator.isFull());
                assertFalse(allocator.isEmpty());

                assertTimeoutPreemptively(Duration.ofSeconds(10), () -> allocator.free(alloc));
            }

            assertTrue(allocator.isEmpty());
            assertFalse(allocator.isFull());
        }
    }
}
