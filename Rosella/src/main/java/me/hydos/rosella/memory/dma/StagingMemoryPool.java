package me.hydos.rosella.memory.dma;

import me.hydos.rosella.memory.allocators.BackedRingAllocator;
import me.hydos.rosella.memory.allocators.HostMappedAllocation;
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

public class StagingMemoryPool {

    private RingAllocator mainPool;

    public StagingMemoryPool(long vmaInstance) {
        this.mainPool = new RingAllocator(1024L * 1024L * 128L, vmaInstance);
    }

    public void destroy() {
        this.mainPool.destroy();
        this.mainPool = null;
    }

    public HostMappedAllocation allocate(long size) {
        return this.mainPool.allocate(size);
    }

    private class RingAllocator {
        public static final long MAX_BUFFER_SIZE = BackedRingAllocator.MAX_BUFFER_SIZE;
        public static final long MIN_BUFFER_SIZE = BackedRingAllocator.MIN_BUFFER_SIZE;

        private BackedRingAllocator allocator;
        private ByteBuffer memory;
        private final long vmaInstance;
        private long vmaAllocation = VK10.VK_NULL_HANDLE;
        private long vkBuffer = VK10.VK_NULL_HANDLE;

        public RingAllocator(final long size, final long vmaInstance) {
            if(!isPowerOf2(size)) {
                throw new IllegalArgumentException("Size must be power of 2 but was " + size);
            }
            if(size < MIN_BUFFER_SIZE) {
                throw new IllegalArgumentException("Size must be greater than " + MIN_BUFFER_SIZE + " but was " + size);
            }
            if(size > MAX_BUFFER_SIZE) {
                throw new IllegalArgumentException("Size must be smaller than " + MAX_BUFFER_SIZE + " but was " + size);
            }

            this.vmaInstance = vmaInstance;

            try(MemoryStack stack = MemoryStack.stackPush()) {
                VkBufferCreateInfo bufferInfo = VkBufferCreateInfo.callocStack(stack);
                bufferInfo.sType(VK10.VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO);
                bufferInfo.size(size);
                bufferInfo.flags(VK10.VK_BUFFER_USAGE_TRANSFER_SRC_BIT | VK10.VK_BUFFER_USAGE_TRANSFER_DST_BIT);
                bufferInfo.sharingMode(VK10.VK_SHARING_MODE_EXCLUSIVE);

                VmaAllocationCreateInfo allocCreateInfo = VmaAllocationCreateInfo.callocStack(stack);
                allocCreateInfo.flags(Vma.VMA_ALLOCATION_CREATE_MAPPED_BIT);
                allocCreateInfo.usage(Vma.VMA_MEMORY_USAGE_CPU_ONLY);
                allocCreateInfo.requiredFlags(VK10.VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK10.VK_MEMORY_PROPERTY_HOST_COHERENT_BIT);

                VmaAllocationInfo allocInfo = VmaAllocationInfo.callocStack(stack);

                LongBuffer pBuffer = stack.longs(0);
                PointerBuffer pAlloc = stack.pointers(0);

                int result = Vma.vmaCreateBuffer(this.vmaInstance, bufferInfo, allocCreateInfo, pBuffer, pAlloc, allocInfo);
                if(result != VK10.VK_SUCCESS) {
                    throw new RuntimeException("Failed to allocate staging memory " + result);
                }

                this.vmaAllocation = pAlloc.get();
                this.vkBuffer = pBuffer.get();

                this.memory = MemoryUtil.memByteBuffer(allocInfo.pMappedData(), (int) size);
                this.allocator = new BackedRingAllocator(memory);
            }
        }

        public void destroy() {
            this.allocator = null;
            this.memory = null;
            if(this.vkBuffer != VK10.VK_NULL_HANDLE) {
                Vma.vmaDestroyBuffer(this.vmaInstance, this.vkBuffer, this.vmaAllocation);
                this.vkBuffer = VK10.VK_NULL_HANDLE;
                this.vmaAllocation = VK10.VK_NULL_HANDLE;
            } else {
                throw new IllegalStateException("Tried to destroy ring allocator twice");
            }
        }

        public StagingMemoryAllocation allocate(long size) {
            if(size > Integer.MAX_VALUE) {
                return null;
            }

            int address = this.allocator.allocate((int) size);
            if(address != Integer.MIN_VALUE) {
                return null;
            }

            ByteBuffer data = this.memory.slice(address, (int) size);
            return new StagingMemoryAllocation(this, address, data);
        }

        public void free(long address) {
            this.allocator.free((int) address);
        }

        private static boolean isPowerOf2(long value) {
            return (value > 0) && ((value & (value-1)) == 0);
        }
    }

    private class StagingMemoryAllocation implements HostMappedAllocation {

        private RingAllocator allocator;
        private ByteBuffer hostBuffer;
        private final long id;
        private final long byteSize;

        public StagingMemoryAllocation(RingAllocator allocator, long id, ByteBuffer hostBuffer) {
            this.allocator = allocator;
            this.hostBuffer = hostBuffer;
            this.id = id;
            this.byteSize = hostBuffer.limit();
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
    }
}
