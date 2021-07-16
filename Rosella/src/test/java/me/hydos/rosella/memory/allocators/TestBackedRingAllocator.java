package me.hydos.rosella.memory.allocators;

import org.junit.jupiter.api.Test;

import java.nio.ByteBuffer;
import java.time.Duration;
import java.util.*;

import static org.junit.jupiter.api.Assertions.*;

public class TestBackedRingAllocator {

    private static BackedRingAllocator create(int size) {
        return new BackedRingAllocator(ByteBuffer.allocate(size));
    }

    @Test
    void createInstance() {
        assertDoesNotThrow(() -> create(16));
        assertDoesNotThrow(() -> create(1024));
        assertDoesNotThrow(() -> create(268435456));

        assertThrows(IllegalArgumentException.class, () -> create(0));
        assertThrows(IllegalArgumentException.class, () -> create(3));
        assertThrows(IllegalArgumentException.class, () -> create(10289));
    }

    @Test
    void allocateFreeSingle() {
        int[] allocationSizesSuccess = new int[]{ 16, 15, 1020, 471, 981, 1};
        for(int size : allocationSizesSuccess) {
            BackedRingAllocator allocator = create(1024);
            assertTrue(allocator.isEmpty());
            assertFalse(allocator.isFull());

            int alloc = allocator.allocate(size);
            assertNotEquals(Integer.MIN_VALUE, alloc, "Allocation failed with " + size);

            assertFalse(allocator.isEmpty(), "Should not be empty when allocating " + size);
            if(size != 1020) {
                assertFalse(allocator.isFull(), "Should not be full when allocating " + size);
            } else {
                assertTrue(allocator.isFull(), "Should be full after allocating 1020");
            }

            assertTimeoutPreemptively(Duration.ofSeconds(10), () -> allocator.free(alloc), "Free should not take over 10s for " + size);

            assertTrue(allocator.isEmpty(), "Should be empty after free " + size);
            assertFalse(allocator.isFull(), "Should not be full after free " + size);
        }

        int[] allocationSizesFail = new int[]{ 1024, 1025, 1021, 38924, Integer.MAX_VALUE };
        for(int size : allocationSizesFail) {
            BackedRingAllocator allocator = create(1024);
            assertTrue(allocator.isEmpty());
            assertFalse(allocator.isFull());

            assertDoesNotThrow(() -> {
                int alloc = allocator.allocate(size);
                assertEquals(Integer.MIN_VALUE, alloc, "Allocation should fail when allocating " + size);

                assertTrue(allocator.isEmpty(), "Should be empty after failed allocation " + size);
                assertFalse(allocator.isFull(), "Should not be full after failed allocation " + size);
            });
        }

        int[] allocationSizesThrow = new int[]{ 0, -3, -1020, -1024, -1025, -349094, Integer.MIN_VALUE };
        for(int size : allocationSizesThrow) {
            BackedRingAllocator allocator = create(1024);
            assertTrue(allocator.isEmpty());
            assertFalse(allocator.isFull());

            assertThrows(IllegalArgumentException.class, () -> allocator.allocate(size));

            assertTrue(allocator.isEmpty(), "Should be empty after failed allocation " + size);
            assertFalse(allocator.isFull(), "Should not be full after failed allocation " + size);
        }
    }

    @Test
    void allocateFreeSeqOrdered() {
        BackedRingAllocator allocator = create(1024);
        Deque<Integer> allocations = new ArrayDeque<>();

        for(int i = 0; i < 32; i++) {
            int alloc = allocator.allocate(i + 1);
            assertNotEquals(Integer.MIN_VALUE, alloc);

            assertFalse(allocator.isEmpty());
            assertFalse(allocator.isFull());

            allocations.addFirst(alloc);
        }

        for(int i = 0; i < 32; i++) {
            int alloc = allocations.pollLast();

            assertFalse(allocator.isEmpty());
            assertFalse(allocator.isFull());

            assertTimeoutPreemptively(Duration.ofSeconds(10), () -> allocator.free(alloc));
        }

        assertTrue(allocator.isEmpty());
        assertFalse(allocator.isFull());
    }

    @Test
    void allocateFreeSeqRandom() {
        Random random = new Random(347982);
        BackedRingAllocator allocator = create(1024);
        List<Integer> allocations = new ArrayList<>();

        for(int i = 0; i < 32; i++) {
            int alloc = allocator.allocate(i + 1);
            assertNotEquals(Integer.MIN_VALUE, alloc);

            assertFalse(allocator.isEmpty());
            assertFalse(allocator.isFull());

            allocations.add(alloc);
        }

        for(int i = 0; i < 32; i++) {
            int alloc = allocations.remove(random.nextInt(allocations.size()));

            assertFalse(allocator.isEmpty());
            assertFalse(allocator.isFull());

            assertTimeoutPreemptively(Duration.ofSeconds(10), () -> allocator.free(alloc));
        }

        assertTrue(allocator.isEmpty());
        assertFalse(allocator.isFull());
    }

    @Test
    void allocateFreeWraparound() {
        BackedRingAllocator allocator = create(1024);
        Deque<Integer> allocations = new ArrayDeque<>();

        allocations.addFirst(allocator.allocate(900));
        allocations.addFirst(allocator.allocate(80));
        allocator.free(allocations.pollLast());

        assertFalse(allocator.isEmpty());
        assertFalse(allocator.isFull());

        int alloc = allocator.allocate(512);
        assertEquals(4, alloc);

        assertFalse(allocator.isEmpty());
        assertFalse(allocator.isFull());

        allocator.free(allocations.pollLast());

        assertFalse(allocator.isEmpty());
        assertFalse(allocator.isFull());

        allocator.free(alloc);

        assertTrue(allocator.isEmpty());
        assertFalse(allocator.isFull());

        alloc = allocator.allocate(1020);
        assertNotEquals(Integer.MIN_VALUE, alloc);

        assertFalse(allocator.isEmpty());
        assertTrue(allocator.isFull());
    }

    @Test
    void allocateFreeFullRandom() {
        Random random = new Random(1848934);
        BackedRingAllocator allocator = create(2048);
        Deque<Integer> allocations = new LinkedList<>();
        List<Integer> tmpShuffle = new ArrayList<>();

        // We will never have more than 16 allocations so 100 ensures the allocator is never full and theres always space for a wraparound
        final int MAX_ALLOC_SIZE = 100 - 1;

        assertTrue(allocator.isEmpty());
        assertFalse(allocator.isFull());

        // At the end of each of these iterations the allocator should be empty
        for(int n = 0; n < 8; n++) {
            // Make 8 initial allocations
            for(int i = 0; i < 8; i++) {
                int alloc = allocator.allocate(random.nextInt(MAX_ALLOC_SIZE) + 1);
                assertNotEquals(Integer.MIN_VALUE, alloc, "Failed to allocate initial " + i + " in iteration " + n);
                allocations.addFirst(alloc);
            }

            assertFalse(allocator.isEmpty());
            assertFalse(allocator.isFull());

            // Run a lot of allocate free cycles
            for(int k = 0; k < 2048; k++) {
                for(int i = 0; i < 8; i++) {
                    int alloc = allocator.allocate(random.nextInt(MAX_ALLOC_SIZE) + 1);
                    assertNotEquals(Integer.MIN_VALUE, alloc, "Failed to allocate " + i + " in batch " + k + " and iteration " + n);
                    allocations.addFirst(alloc);
                }

                // Remove last 8 and randomly shuffle them
                tmpShuffle.clear();
                for(int i = 0; i < 8; i++) {
                    tmpShuffle.add(allocations.pollLast());
                    Collections.shuffle(tmpShuffle, random);
                }
                for(int alloc : tmpShuffle) {
                    assertTimeoutPreemptively(Duration.ofSeconds(10), () -> allocator.free(alloc), "Failed to free in batch " + k + " and iteration " + n);
                }

                assertFalse(allocator.isEmpty());
                assertFalse(allocator.isFull());
            }

            // Free the remaining 8 allocations again
            for(int alloc : allocations) {
                assertTimeoutPreemptively(Duration.ofSeconds(10), () -> allocator.free(alloc), "Failed to free final in iteration " + n);
            }
            allocations.clear();

            assertTrue(allocator.isEmpty());
            assertFalse(allocator.isFull());

            // Make sure the empty allocator can always make a full allocation
            int endAlloc = allocator.allocate(2044);

            assertNotEquals(Integer.MIN_VALUE, endAlloc);
            assertFalse(allocator.isEmpty());
            assertTrue(allocator.isFull());

            allocator.free(endAlloc);

            assertTrue(allocator.isEmpty());
            assertFalse(allocator.isFull());
        }
    }
}
