package me.hydos.rosella.memory.dma;

import me.hydos.rosella.memory.allocators.BackedRingAllocator;
import me.hydos.rosella.memory.allocators.HostMappedAllocation;
import me.hydos.rosella.memory.allocators.UnbackedRingAllocator;
import org.lwjgl.PointerBuffer;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.system.MemoryUtil;
import org.lwjgl.util.vma.Vma;
import org.lwjgl.util.vma.VmaAllocationCreateInfo;
import org.lwjgl.util.vma.VmaAllocationInfo;
import org.lwjgl.vulkan.*;

import java.nio.ByteBuffer;
import java.nio.LongBuffer;
import java.util.concurrent.locks.Lock;
import java.util.concurrent.locks.ReentrantLock;

public class StagingMemoryPool {

    private final long vmaAllocator;
    private RingAllocator mainPool;

    public StagingMemoryPool(long vmaAllocator) {
        this.vmaAllocator = vmaAllocator;
        this.mainPool = new RingAllocator(1024L * 1024L * 128L);
    }

    public void destroy() {
        this.mainPool.destroy();
        this.mainPool = null;
    }

    public StagingMemoryAllocation allocate(long size) {
        return this.mainPool.allocate(size);
    }

    private class RingAllocator {
        public static final long MAX_BUFFER_SIZE = BackedRingAllocator.MAX_BUFFER_SIZE;
        public static final long MIN_BUFFER_SIZE = BackedRingAllocator.MIN_BUFFER_SIZE;

        private UnbackedRingAllocator allocator;
        private ByteBuffer memory;
        private long vmaAllocation = VK10.VK_NULL_HANDLE;
        private long vkBuffer = VK10.VK_NULL_HANDLE;

        private final Lock lock = new ReentrantLock();

        public RingAllocator(final long size) {
            if(!isPowerOf2(size)) {
                throw new IllegalArgumentException("Size must be power of 2 but was " + size);
            }
            if(size < MIN_BUFFER_SIZE) {
                throw new IllegalArgumentException("Size must be greater than " + MIN_BUFFER_SIZE + " but was " + size);
            }
            if(size > MAX_BUFFER_SIZE) {
                throw new IllegalArgumentException("Size must be smaller than " + MAX_BUFFER_SIZE + " but was " + size);
            }

            try(MemoryStack stack = MemoryStack.stackPush()) {
                VkBufferCreateInfo bufferInfo = VkBufferCreateInfo.callocStack(stack);
                bufferInfo.sType(VK10.VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO);
                bufferInfo.size(size);
                bufferInfo.usage(VK10.VK_BUFFER_USAGE_TRANSFER_SRC_BIT | VK10.VK_BUFFER_USAGE_TRANSFER_DST_BIT);
                bufferInfo.sharingMode(VK10.VK_SHARING_MODE_EXCLUSIVE);

                VmaAllocationCreateInfo allocCreateInfo = VmaAllocationCreateInfo.callocStack(stack);
                allocCreateInfo.flags(Vma.VMA_ALLOCATION_CREATE_MAPPED_BIT);
                allocCreateInfo.usage(Vma.VMA_MEMORY_USAGE_CPU_ONLY);
                allocCreateInfo.requiredFlags(VK10.VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK10.VK_MEMORY_PROPERTY_HOST_COHERENT_BIT);

                VmaAllocationInfo allocInfo = VmaAllocationInfo.callocStack(stack);

                LongBuffer pBuffer = stack.longs(0);
                PointerBuffer pAlloc = stack.pointers(0);

                int result = Vma.vmaCreateBuffer(vmaAllocator, bufferInfo, allocCreateInfo, pBuffer, pAlloc, allocInfo);
                if(result != VK10.VK_SUCCESS) {
                    throw new RuntimeException("Failed to allocate staging memory " + result);
                }

                this.vmaAllocation = pAlloc.get();
                this.vkBuffer = pBuffer.get();

                this.memory = MemoryUtil.memByteBuffer(allocInfo.pMappedData(), (int) size);
                this.allocator = new UnbackedRingAllocator(memory.remaining());
            }
        }

        public void destroy() {
            this.allocator = null;
            this.memory = null;
            if(this.vkBuffer != VK10.VK_NULL_HANDLE) {
                Vma.vmaDestroyBuffer(vmaAllocator, this.vkBuffer, this.vmaAllocation);
                this.vkBuffer = VK10.VK_NULL_HANDLE;
                this.vmaAllocation = VK10.VK_NULL_HANDLE;
            } else {
                throw new IllegalStateException("Tried to destroy ring allocator twice");
            }
        }

        public StagingMemoryAllocation allocate(long size) {
            if (size > Integer.MAX_VALUE) {
                return null;
            }

            try {
                lock.lock();
                int address = (int) this.allocator.allocate(size);
                if (address == -1) {
                    return null;
                }

                ByteBuffer data = this.memory.slice(address, (int) size);
                return new StagingMemoryAllocation(this, address, data, this.vkBuffer);
            } finally {
                lock.unlock();
            }
        }

        public void free(long address) {
            try {
                lock.lock();
                this.allocator.free(address);
            } finally {
                lock.unlock();
            }
        }

        private static boolean isPowerOf2(long value) {
            return (value > 0) && ((value & (value-1)) == 0);
        }
    }

    public class StagingMemoryAllocation implements HostMappedAllocation {

        private RingAllocator allocator;
        private ByteBuffer hostBuffer;
        private final long id;
        private final long byteSize;
        private final long vkBuffer;

        public StagingMemoryAllocation(RingAllocator allocator, long id, ByteBuffer hostBuffer, long vkBuffer) {
            this.allocator = allocator;
            this.hostBuffer = hostBuffer;
            this.id = id;
            this.byteSize = hostBuffer.limit();
            this.vkBuffer = vkBuffer;
        }

        @Override
        public long getByteSize() {
            return this.byteSize;
        }

        @Override
        public void free() {
            if(this.allocator != null) {
                this.allocator.free(this.id);
                this.allocator = null;
                this.hostBuffer = null;
            }
        }

        @Override
        public ByteBuffer getHostBuffer() {
            return this.hostBuffer;
        }

        public long getBufferOffset() {
            return this.id;
        }

        public long getVulkanBuffer() {
            return this.vkBuffer;
        }
    }
}
