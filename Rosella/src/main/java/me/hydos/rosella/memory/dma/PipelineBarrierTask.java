package me.hydos.rosella.memory.dma;

import it.unimi.dsi.fastutil.objects.ObjectArraySet;
import me.hydos.rosella.Rosella;
import org.lwjgl.PointerBuffer;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.vulkan.VK10;
import org.lwjgl.vulkan.VkBufferMemoryBarrier;
import org.lwjgl.vulkan.VkMemoryBarrier;

import java.util.Objects;
import java.util.Set;

public class PipelineBarrierTask extends Task {

    private int srcStageMask = 0;
    private int dstStageMask = 0;

    private final Set<MemoryBarrier> memoryBarriers = new ObjectArraySet<>();
    private final Set<BufferMemoryBarrier> bufferMemoryBarriers = new ObjectArraySet<>();

    public PipelineBarrierTask(int srcStageMask, int dstStageMask) {
        this.srcStageMask = srcStageMask;
        this.dstStageMask = dstStageMask;
    }

    public PipelineBarrierTask addMemoryBarrier(int srcAccessMask, int dstAccessMask) {
        addMemoryBarrier(new MemoryBarrier(srcAccessMask, dstAccessMask));
        return this;
    }

    public PipelineBarrierTask addBufferMemoryBarrier(int srcAccessMask, int dstAccessMask, int srcQueue, int dstQueue, long buffer, long offset, long size) {
        addBufferMemoryBarrier(new BufferMemoryBarrier(srcAccessMask, dstAccessMask, srcQueue, dstQueue, buffer, offset, size));
        return this;
    }

    private void addMemoryBarrier(MemoryBarrier barrier) {
        memoryBarriers.add(barrier);
    }

    private void addBufferMemoryBarrier(BufferMemoryBarrier barrier) {
        bufferMemoryBarriers.add(barrier);
    }

    @Override
    public boolean canReorderBehind(Task other) {
        return false;
    }

    @Override
    public Task tryMergeWith(Task o) {
        if(getClass() != o.getClass()) {
            return null;
        }

        PipelineBarrierTask other = (PipelineBarrierTask) o;

        this.srcStageMask |= other.srcStageMask;
        this.dstStageMask |= other.dstStageMask;

        this.memoryBarriers.addAll(other.memoryBarriers);
        this.bufferMemoryBarriers.addAll(other.bufferMemoryBarriers);
        return this;
    }

    @Override
    public void record(DMARecorder recorder) {
        Rosella.LOGGER.warn("Recording barrier task");

        try (MemoryStack stack = MemoryStack.stackPush()) {
            VkMemoryBarrier.Buffer memoryBarrierBuffer = null;
            if(this.memoryBarriers.size() != 0) {
                memoryBarrierBuffer = VkMemoryBarrier.mallocStack(this.memoryBarriers.size(), stack);
                for(MemoryBarrier barrier : this.memoryBarriers) {
                    barrier.fillStructure(memoryBarrierBuffer.get());
                }
                memoryBarrierBuffer.rewind();
            }

            VkBufferMemoryBarrier.Buffer bufferMemoryBarrierBuffer = null;
            if(this.bufferMemoryBarriers.size() != 0) {
                bufferMemoryBarrierBuffer = VkBufferMemoryBarrier.mallocStack(this.bufferMemoryBarriers.size(), stack);
                for(BufferMemoryBarrier barrier : this.bufferMemoryBarriers) {
                    barrier.fillStructure(bufferMemoryBarrierBuffer.get());
                }
                bufferMemoryBarrierBuffer.rewind();
            }

            VK10.vkCmdPipelineBarrier(
                    recorder.getCommandBuffer(),
                    this.srcStageMask, this.dstStageMask, 0,
                    memoryBarrierBuffer,
                    bufferMemoryBarrierBuffer,
                    null);
        }
    }

    private static record MemoryBarrier(int srcAccessMask, int dstAccessMask) {
        public void fillStructure(VkMemoryBarrier structure) {
            structure.sType(VK10.VK_STRUCTURE_TYPE_MEMORY_BARRIER);
            structure.srcAccessMask(this.srcAccessMask);
            structure.dstAccessMask(this.dstAccessMask);
        }

        @Override
        public boolean equals(Object o) {
            if (this == o) return true;
            if (o == null || getClass() != o.getClass()) return false;
            MemoryBarrier that = (MemoryBarrier) o;
            return srcAccessMask == that.srcAccessMask && dstAccessMask == that.dstAccessMask;
        }

        @Override
        public int hashCode() {
            return Objects.hash(srcAccessMask, dstAccessMask);
        }
    }

    private static class BufferMemoryBarrier {
        private int srcAccessMask;
        private int dstAccessMask;
        private int srcQueue;
        private int dstQueue;
        private long buffer;
        private long offset;
        private long size;

        public BufferMemoryBarrier(int srcAccessMask, int dstAccessMask, int srcQueue, int dstQueue, long buffer, long offset, long size) {
            this.srcAccessMask = srcAccessMask;
            this.dstAccessMask = dstAccessMask;
            this.srcQueue = srcQueue;
            this.dstQueue = dstQueue;
            this.buffer = buffer;
            this.offset = offset;
            this.size = size;
        }

        public void fillStructure(VkBufferMemoryBarrier structure) {
            structure.sType(VK10.VK_STRUCTURE_TYPE_BUFFER_MEMORY_BARRIER);
            structure.pNext(VK10.VK_NULL_HANDLE);
            structure.srcAccessMask(this.srcAccessMask);
            structure.dstAccessMask(this.dstAccessMask);
            structure.srcQueueFamilyIndex(this.srcQueue);
            structure.dstQueueFamilyIndex(this.dstQueue);
            structure.buffer(this.buffer);
            structure.offset(this.offset);
            structure.size(this.size);
        }

        @Override
        public boolean equals(Object o) {
            if (this == o) return true;
            if (o == null || getClass() != o.getClass()) return false;
            BufferMemoryBarrier that = (BufferMemoryBarrier) o;
            return srcAccessMask == that.srcAccessMask && dstAccessMask == that.dstAccessMask && srcQueue == that.srcQueue && dstQueue == that.dstQueue && buffer == that.buffer && offset == that.offset && size == that.size;
        }

        @Override
        public int hashCode() {
            return Objects.hash(srcAccessMask, dstAccessMask, srcQueue, dstQueue, buffer, offset, size);
        }
    }
}
