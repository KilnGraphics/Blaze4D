package me.hydos.rosella.memory.dma;

import org.lwjgl.PointerBuffer;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.system.MemoryUtil;
import org.lwjgl.util.vma.Vma;
import org.lwjgl.util.vma.VmaAllocationCreateInfo;
import org.lwjgl.util.vma.VmaAllocationInfo;
import org.lwjgl.vulkan.VK10;
import org.lwjgl.vulkan.VkBufferCreateInfo;

import java.nio.ByteBuffer;
import java.nio.LongBuffer;
import java.util.ArrayList;
import java.util.List;
import java.util.Random;
import java.util.concurrent.locks.Lock;
import java.util.concurrent.locks.ReentrantLock;

public class StagingMemoryPool {

    private static final long INVALID_ADDRESS = Long.MIN_VALUE;

    private final long vmaAllocator;
    private final RingBuffer mainBuffer;

    public StagingMemoryPool(long vmaAllocator) {
        this.vmaAllocator = vmaAllocator;
        this.mainBuffer = new RingBuffer(1024L * 1024L * 256L);
    }

    public void destroy() {
        this.mainBuffer.destroy();
    }

    public Allocation allocate(long size) {
        Allocation allocation = mainBuffer.allocate(size);
        if(allocation == null) {
            throw new RuntimeException("Out of ring buffer memory");
        }
        return allocation;
    }

    public class Allocation {
        private final ByteBuffer dataBuffer;
        private final long dataOffset;
        private final long dataSize;
        private final long internalOffset;
        private final RingBuffer source;

        public Allocation(RingBuffer source, long internalOffset, long dataOffset, long dataSize, ByteBuffer dataBuffer) {
            this.dataBuffer = dataBuffer;
            this.dataOffset = dataOffset;
            this.dataSize = dataSize;
            this.internalOffset = internalOffset;
            this.source = source;
        }

        public ByteBuffer getData() {
            return this.dataBuffer;
        }

        public long getOffset() {
            return this.dataOffset;
        }

        public long getSize() {
            return this.dataSize;
        }

        public long getVulkanBuffer() {
            return this.source.vkBuffer;
        }

        public void free() {
            this.source.free(internalOffset);
        }
    }

    private class RingBuffer {
        public static final long MAX_BUFFER_SIZE = Integer.MAX_VALUE; // Need to make sure we never get negative ints because java is pain
        private static final long HEADER_SIZE = 4;

        private final long bufferSize;
        private final ByteBuffer mappedMemory;

        public final boolean requiresFlush;

        private final long vkBuffer;
        private final long vmaAllocation;

        private long virtualHeadOffset = 0;
        private long virtualTailOffset = 0;

        private final Lock lock = new ReentrantLock();

        public RingBuffer(long size) {
            if(size > MAX_BUFFER_SIZE) {
                throw new IllegalArgumentException("Size (" + size + ") exceeds maximum allowed buffer size of " + MAX_BUFFER_SIZE);
            }

            if(!isPowerOf2(size)) {
                throw new IllegalArgumentException("Size must be power of 2 but was: " + size);
            }

            this.bufferSize = size;

            try(MemoryStack stack = MemoryStack.stackPush()) {
                VkBufferCreateInfo bufferInfo = VkBufferCreateInfo.callocStack(stack);
                bufferInfo.sType(VK10.VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO);
                bufferInfo.size(this.bufferSize);
                bufferInfo.usage(VK10.VK_BUFFER_USAGE_TRANSFER_SRC_BIT | VK10.VK_BUFFER_USAGE_TRANSFER_DST_BIT);
                bufferInfo.sharingMode(VK10.VK_SHARING_MODE_EXCLUSIVE);

                VmaAllocationCreateInfo allocCreateInfo = VmaAllocationCreateInfo.callocStack(stack);
                allocCreateInfo.usage(Vma.VMA_MEMORY_USAGE_CPU_ONLY);
                allocCreateInfo.flags(Vma.VMA_ALLOCATION_CREATE_MAPPED_BIT);
                allocCreateInfo.requiredFlags(VK10.VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK10.VK_MEMORY_PROPERTY_HOST_COHERENT_BIT);
                //allocCreateInfo.preferredFlags();

                VmaAllocationInfo allocInfo = VmaAllocationInfo.callocStack(stack);

                LongBuffer pBuffer = stack.longs(0);
                PointerBuffer pAllocation = stack.pointers(0);
                int result = Vma.vmaCreateBuffer(vmaAllocator, bufferInfo, allocCreateInfo, pBuffer, pAllocation, allocInfo);
                if(result != VK10.VK_SUCCESS) {
                    throw new RuntimeException("Failed to allocate staging buffer memory");
                }

                this.vkBuffer = pBuffer.get();
                this.vmaAllocation = pAllocation.get();
                this.mappedMemory = MemoryUtil.memByteBuffer(allocInfo.pMappedData(), (int) size);
                this.requiresFlush = false;
            }
        }

        public void destroy() {
            try {
                lock();
                if(this.virtualTailOffset != this.virtualHeadOffset) {
                    // TODO warn
                }

                Vma.vmaDestroyBuffer(vmaAllocator, this.vkBuffer, this.vmaAllocation);
            } finally {
                unlock();
            }
        }

        public Allocation allocate(final long size) {
            final long fullSize = ((size + 3) & (~3)); // Round up to multiple of 4
            if(fullSize + HEADER_SIZE > this.bufferSize) {
                return null;
            }

            Allocation result = null;
            try {
                lock();
                long virtualFit = findVirtualFit(fullSize);
                if(virtualFit == INVALID_ADDRESS) {
                    return null;
                }

                final long newVirtualHead = virtualFit + fullSize + HEADER_SIZE;
                final long realBlockStart = getRealOffset(this.virtualHeadOffset);
                final long realDataStart = getRealOffset(virtualFit);

                final long internalBlockSize = newVirtualHead - this.virtualHeadOffset;
                writeBlockHeader((int) realBlockStart, (int) internalBlockSize);

                ByteBuffer dataBuffer = mappedMemory.slice((int) realDataStart, (int) size);
                result = new Allocation(this, this.virtualHeadOffset, realDataStart, size, dataBuffer);

                this.virtualHeadOffset = newVirtualHead;
            } finally {
                unlock();
            }

            return  result;
        }

        public void free(final long offset) {
            final long realOffset = getRealOffset(offset);
            try {
                lock();
                markBlockEmpty((int) realOffset);
                chainFreeBlocks();
            } finally {
                unlock();
            }
        }

        private void chainFreeBlocks() {
            long currentBlockSize;
            while((currentBlockSize = getBlockSizeIfEmpty((int) getRealOffset(this.virtualTailOffset))) != INVALID_ADDRESS) {
                this.virtualTailOffset += currentBlockSize;
            }
        }

        private void writeBlockHeader(final int realPosition, final int blockSize) {
            final int headerContent = blockSize & (~3); // Clear first 2 bits
            mappedMemory.putInt(realPosition, headerContent);
        }

        private void markBlockEmpty(final int realPosition) {
            int currentValue = mappedMemory.getInt(realPosition);
            mappedMemory.putInt(realPosition, currentValue | 1);
        }

        private long getBlockSizeIfEmpty(final int realHeaderPosition) {
            final int header = mappedMemory.getInt(realHeaderPosition);
            if((header & 1) == 1) {
                return header & ~1;
            } else {
                return INVALID_ADDRESS;
            }
        }

        /**
         * @param size The block size not including the header. Must be a multiple of 4
         * @return The virtual address of the beginning of a block that satisfies the allocation requirements
         *         or <code>INVALID_ADDRESS</code> if no such block exists.
         */
        private long findVirtualFit(long size) {
            if(this.isFull()) {
                System.out.println("IS FULL");
                return INVALID_ADDRESS;
            }

            size += HEADER_SIZE;
            final long realHead = getRealOffset(this.virtualHeadOffset);
            final long realTail = getRealOffset(this.virtualTailOffset);

            long result = INVALID_ADDRESS;
            if(realHead < realTail) {
                if(realTail - realHead >= size) {
                    result = this.virtualHeadOffset;
                }
            } else {
                if(this.bufferSize - realHead >= size) {
                    result = this.virtualHeadOffset;
                } else {
                    // TODO: we can remove the header size from the size here if there is enough space at the end of the buffer. But we need to tell that the callee somehow
                    if(realTail >= size) {
                        result = this.virtualHeadOffset + (this.bufferSize - realHead);
                    }
                }
            }
            return result;
        }

        private boolean isFull() {
            return (this.virtualHeadOffset - this.virtualTailOffset) == this.bufferSize;
        }

        private long getRealOffset(final long virtualOffset) {
            return virtualOffset & (this.bufferSize - 1);
        }

        private void lock() {
            this.lock.lock();
        }

        private void unlock() {
            this.lock.unlock();
        }

        private static boolean isPowerOf2(long value) {
            return (value > 0L) && (value & value-1) == 0;
        }
    }

    public void randomTests() {
        Random rand = new Random();
        List<Allocation> allocations = new ArrayList<>();
        Allocation allocation = null;

        System.out.println("Running staging memory pool tests");

        System.out.println("Allocate - Free:");
        allocation = this.allocate(1020L);
        assert(this.mainBuffer.virtualHeadOffset == 1024L);
        assert(this.mainBuffer.getBlockSizeIfEmpty(0) == INVALID_ADDRESS);

        allocation.free();
        assert(this.mainBuffer.virtualHeadOffset == 1024L);
        assert(this.mainBuffer.virtualTailOffset == 1024L);

        System.out.println("Allocate - Free unaligned:");
        allocation = this.allocate(1019L);
        assert(this.mainBuffer.virtualHeadOffset == 2048L);
        assert(this.mainBuffer.getBlockSizeIfEmpty(0) == INVALID_ADDRESS);

        allocation.free();
        assert(this.mainBuffer.virtualHeadOffset == 2048L);
        assert(this.mainBuffer.virtualTailOffset == 2048L);

        System.out.println("Allocate - Free a lot:");
        for(int i = 0; i < 128; i++) {
            allocations.add(this.allocate(1020L));
        }
        for(Allocation alloc : allocations) {
            alloc.free();
        }
        allocations.clear();
        assert(this.mainBuffer.virtualHeadOffset == 2048L + (1024L * 128L));
        assert(this.mainBuffer.virtualTailOffset == 2048L + (1024L * 128L));

        System.out.println("Allocate - Free a lot unaligned:");
        for(int i = 0; i < 128; i++) {
            allocations.add(this.allocate(1017L));
        }
        for(Allocation alloc : allocations) {
            alloc.free();
        }
        allocations.clear();
        assert(this.mainBuffer.virtualHeadOffset == 2048L + (1024L * 128L * 2L));
        assert(this.mainBuffer.virtualTailOffset == 2048L + (1024L * 128L * 2L));

        System.out.println("Allocate - Free random:");
        for(int i = 0; i < 128; i++) {
            allocations.add(this.allocate(1020L));
        }
        while(!allocations.isEmpty()) {
            allocations.remove(rand.nextInt(allocations.size())).free();
        }

        assert(this.mainBuffer.virtualHeadOffset == 2048L + (1024L * 128L * 3L));
        assert(this.mainBuffer.virtualTailOffset == 2048L + (1024L * 128L * 3L));

        System.out.println("Allocate - Free random unaligned:");
        for(int i = 0; i < 128; i++) {
            allocations.add(this.allocate(1018L));
        }
        while(!allocations.isEmpty()) {
            allocations.remove(rand.nextInt(allocations.size())).free();
        }

        assert(this.mainBuffer.virtualHeadOffset == 2048L + (1024L * 128L * 4L));
        assert(this.mainBuffer.virtualTailOffset == 2048L + (1024L * 128L * 4L));
    }
}
