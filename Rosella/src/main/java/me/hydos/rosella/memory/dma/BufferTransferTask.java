package me.hydos.rosella.memory.dma;

import it.unimi.dsi.fastutil.objects.ObjectArraySet;
import me.hydos.rosella.Rosella;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.vulkan.VK10;
import org.lwjgl.vulkan.VkBufferCopy;

import java.util.Objects;
import java.util.Set;

public class BufferTransferTask extends Task {

    private final long srcBuffer;
    private final long dstBuffer;
    private Set<BufferRegion> regions = new ObjectArraySet<>();

    public BufferTransferTask(long srcBuffer, long dstBuffer) {
        this.srcBuffer = srcBuffer;
        this.dstBuffer = dstBuffer;
    }

    public BufferTransferTask addRegion(long srcOffset, long dstOffset, long size) {
        regions.add(new BufferRegion(srcOffset, dstOffset, size));
        return this;
    }

    @Override
    public void record(DMARecorder recorder) {
        if(regions.size() == 0) {
            return;
        }

        try(MemoryStack stack = MemoryStack.stackPush()) {
            VkBufferCopy.Buffer regionBuffer = VkBufferCopy.mallocStack(regions.size(), stack);
            for(BufferRegion region : regions) {
                region.fillStructure(regionBuffer.get());
            }
            regionBuffer.rewind();

            VK10.vkCmdCopyBuffer(recorder.getCommandBuffer(), this.srcBuffer, this.dstBuffer, regionBuffer);
        }
    }

    private record BufferRegion(long srcOffset, long dstOffset, long size) {
        public void fillStructure(VkBufferCopy structure) {
            structure.srcOffset(this.srcOffset);
            structure.dstOffset(this.dstOffset);
            structure.size(this.size);
        }

        @Override
        public boolean equals(Object o) {
            if (this == o) return true;
            if (o == null || getClass() != o.getClass()) return false;
            BufferRegion that = (BufferRegion) o;
            return srcOffset == that.srcOffset && dstOffset == that.dstOffset && size == that.size;
        }

        @Override
        public int hashCode() {
            return Objects.hash(srcOffset, dstOffset, size);
        }
    }
}
