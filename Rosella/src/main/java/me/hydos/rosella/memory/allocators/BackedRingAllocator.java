package me.hydos.rosella.memory.allocators;

import java.nio.ByteBuffer;

/**
 * Class that provides a ring buffer allocator.
 *
 * Free operations may happen out of order however the oldest not freed allocation will block new allocations.
 *
 * The allocator will use non allocated parts of the memory to store metadata so the memory must not
 * be accessed outside of allocated regions. The allocator will not free the memory after it is no longer needed.
 *
 * All calls to this class must be externally synchronized.
 *
 * It is generally expected that this class wont be directly accessed by client code but used as a utility to
 * implement a higher level allocator.
 */
public class BackedRingAllocator {
    private static final long INVALID_ADDRESS = Long.MIN_VALUE;
    private static final int HEADER_SIZE = 4;

    public static final long MAX_BUFFER_SIZE = Integer.MAX_VALUE;
    public static final long MIN_BUFFER_SIZE = HEADER_SIZE + 1L;

    private final int memorySize;
    private final ByteBuffer memory;

    private long virtualHead = 0;
    private long virtualTail = 0;

    /**
     * Creates a new ring buffer using the provided memory. The memory must not be changed outside of
     * regions allocated through this class as the memory will be used to store metadata.
     *
     * @param memory The memory used for the allocator. Size must be a power of 2
     */
    public BackedRingAllocator(ByteBuffer memory) {
        this.memorySize = memory.limit();
        this.memory = memory.slice();

        if(this.memorySize < 0) {
            throw new IllegalArgumentException("Invalid memory size " + this.memorySize);
        }
        if(this.memorySize < MIN_BUFFER_SIZE) {
            throw new IllegalArgumentException("Provided memory is too small (" + this.memorySize + ") must be at least " + MIN_BUFFER_SIZE);
        }
        if(!isPowerOf2(this.memorySize)) {
            throw new IllegalArgumentException("Provided memory must be power of 2 but was " + this.memorySize);
        }
    }

    public boolean isEmpty() {
        return this.virtualHead == this.virtualTail;
    }

    public boolean isFull() {
        return (this.virtualHead - this.virtualTail) == this.memorySize;
    }

    public boolean canAllocate(final int size) {
        if(size <= 0) {
            throw new IllegalArgumentException("Size must be greater than 0 but was " + size);
        }
        return findVirtualFit((((long) size) + 4L) & -4L) != INVALID_ADDRESS;
    }

    public int allocate(final int size) {
        if(size <= 0) {
            throw new IllegalArgumentException("Size must be greater than 0 but was " + size);
        }

        final long alignedSize = (((long) size) + 3L) & -4L;

        long virtualFit = findVirtualFit(alignedSize);
        if(virtualFit == INVALID_ADDRESS) {
            return Integer.MIN_VALUE;
        }

        writeAllocation(virtualFit, alignedSize);
        return (int) getRealOffset(virtualFit);
    }

    public void free(final int allocation) {
        // We add the memory size to ensure that a allocation at the beginning of the pool doesnt cause negative numbers when subtracting the header size
        final long virtualPosition = ((long) allocation) + this.memorySize;

        markBlockEmpty(virtualPosition - HEADER_SIZE);
        chainFreeBlocks();
        if(this.virtualTail == this.virtualHead) {
            // If were empty move the head and tail back to the start to allow usage of the full memory
            this.virtualHead = nextWraparound(this.virtualHead);
            this.virtualTail = this.virtualHead;
        }

        if(this.virtualTail > this.virtualHead) {
            throw new IllegalStateException("Ring allocator corruption, tail is in front of head. THIS IS VERY VERY BAD.");
        }
    }

    private void writeAllocation(final long virtualDataStart, final long dataSize) {
        final long virtualBlockStart = virtualDataStart - HEADER_SIZE;
        if(virtualBlockStart != this.virtualHead) {
            // Padding was added so we need to insert a empty block
            writeBlockHeader(this.virtualHead, (int) (virtualBlockStart - this.virtualHead), true);
        }

        writeBlockHeader(virtualBlockStart, (int) (dataSize + HEADER_SIZE), false);
        this.virtualHead = virtualDataStart + dataSize;
    }

    private void chainFreeBlocks() {
        long currentBlockSize;
        while((currentBlockSize = getBlockSizeIfEmpty(this.virtualTail)) != INVALID_ADDRESS) {
            this.virtualTail += currentBlockSize;
            if(this.virtualTail >= this.virtualHead) {
                return;
            }
        }
    }

    /**
     * Calculates the address of the next block of provided size.
     * The passed size should not include header size.
     * The returned value is the virtual start of the data block. The full allocated block starts earlier than
     * this due to the added header.
     *
     * @param size The amount of space to reserve for data not including block metadata. Must be a multiple of 4 and not negative
     * @return The virtual offset of the data block.
     */
    private long findVirtualFit(long size) {
        if(this.isFull()) {
            return INVALID_ADDRESS;
        }

        final long fullSize = size + HEADER_SIZE;
        final long realHead = getRealOffset(this.virtualHead);
        final long realTail = getRealOffset(this.virtualTail);

        long result = INVALID_ADDRESS;
        if(realHead < realTail) {
            if(realTail - realHead >= fullSize) {
                result = this.virtualHead + HEADER_SIZE;
            }
        } else {
            if(this.memorySize - realHead >= fullSize) {
                // There is enough space before a wraparound
                result = this.virtualHead + HEADER_SIZE;

            } else {
                if(realTail >= fullSize) {
                    result = nextWraparound(this.virtualHead) + HEADER_SIZE;
                }
            }
        }

        return result;
    }

    private void writeBlockHeader(final long virtualPosition, final int blockSize, boolean empty) {
        int headerContent = blockSize & (~3);
        if(empty) {
            headerContent = headerContent | 1;
        }
        memory.putInt((int) getRealOffset(virtualPosition), headerContent);
    }

    private void markBlockEmpty(final long virtualPosition) {
        final int realPosition = (int) getRealOffset(virtualPosition);
        int currentHeader = memory.getInt(realPosition);
        memory.putInt(realPosition, currentHeader | 1);
    }

    private long getBlockSizeIfEmpty(final long virtualPosition) {
        final int header = memory.getInt((int) getRealOffset(virtualPosition));
        if((header & 1) == 1) {
            return header & ~3;
        } else {
            return INVALID_ADDRESS;
        }
    }

    /**
     * Rounds up to the next multiple of the memory size.
     */
    private long nextWraparound(final long virtualOffset) {
        return (virtualOffset + this.memorySize) & -this.memorySize;
    }

    private long getRealOffset(final long virtualOffset) {
        return virtualOffset & (this.memorySize - 1);
    }

    private static boolean isPowerOf2(int value) {
        return (value > 0) && ((value & (value-1)) == 0);
    }

    public void testPrint() {
        int tailData = memory.getInt((int) getRealOffset(this.virtualTail));
        System.out.println("Head: " + this.virtualHead + " Tail: " + this.virtualTail + " Data: " + tailData);
    }
}
