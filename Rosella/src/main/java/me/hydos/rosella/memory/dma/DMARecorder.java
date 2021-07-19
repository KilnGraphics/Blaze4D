package me.hydos.rosella.memory.dma;

import it.unimi.dsi.fastutil.longs.LongArraySet;
import it.unimi.dsi.fastutil.longs.LongOpenHashSet;
import it.unimi.dsi.fastutil.objects.ObjectOpenHashSet;
import me.hydos.rosella.Rosella;
import org.jetbrains.annotations.NotNull;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.vulkan.*;

import java.util.ArrayList;
import java.util.List;
import java.util.Set;

public class DMARecorder {

    private VkCommandBuffer commandBuffer;

    private final Set<Task> signalTasks = new ObjectOpenHashSet<>();

    private final Set<Long> acquiredBuffers = new LongArraySet();
    private final Set<Long> releasedBuffers = new LongArraySet();
    private final Set<Long> bufferWrites = new LongOpenHashSet();
    private final Set<Long> bufferReads = new LongOpenHashSet();

    private final List<BufferAcquireTask> acquireTasks = new ArrayList<>();
    private final List<BufferReleaseTask> releaseTasks = new ArrayList<>();

    private final Set<Long> waitSemaphores = new LongArraySet();
    private final Set<Long> signalSemaphores = new LongArraySet();

    public DMARecorder() {
    }

    public void beginRecord(@NotNull VkCommandBuffer commandBuffer) {
        this.commandBuffer = commandBuffer;
        try (MemoryStack stack = MemoryStack.stackPush()) {
            VkCommandBufferBeginInfo beginInfo = VkCommandBufferBeginInfo.callocStack(stack);
            beginInfo.sType(VK10.VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO);
            beginInfo.flags(VK10.VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT);

            int result = VK10.vkBeginCommandBuffer(this.commandBuffer, beginInfo);
            if(result != VK10.VK_SUCCESS) {
                throw new RuntimeException("Failed to begin recoding of transfer command buffer " + result);
            }

            VkBufferMemoryBarrier.Buffer bufferBarriers = null;
            if(!this.acquireTasks.isEmpty()) {
                int i = 0;
                bufferBarriers = VkBufferMemoryBarrier.mallocStack(this.acquireTasks.size(), stack);
                for (BufferAcquireTask task : this.acquireTasks) {
                    task.fillBarrier(bufferBarriers.get(i));
                    i++;
                }
            }

            if(bufferBarriers != null) {
                VK10.vkCmdPipelineBarrier(this.commandBuffer, 0, VK10.VK_PIPELINE_STAGE_TRANSFER_BIT, 0,
                        null, bufferBarriers, null);
            }
        }
    }

    public void endRecord() {
        try (MemoryStack stack = MemoryStack.stackPush()) {
            VkBufferMemoryBarrier.Buffer bufferBarriers = null;
            if(!this.releaseTasks.isEmpty()) {
                int i = 0;
                bufferBarriers = VkBufferMemoryBarrier.mallocStack(this.releaseTasks.size(), stack);
                for (BufferReleaseTask task : this.releaseTasks) {
                    task.fillBarrier(bufferBarriers.get(i));
                    i++;
                }
            }

            if(bufferBarriers != null) {
                VK10.vkCmdPipelineBarrier(this.commandBuffer, VK10.VK_PIPELINE_STAGE_TRANSFER_BIT, 0, 0,
                        null, bufferBarriers, null);
            }
        }

        int result = VK10.vkEndCommandBuffer(this.commandBuffer);
        if(result != VK10.VK_SUCCESS) {
            throw new RuntimeException("Failed to end transfer command buffer recording " + result);
        }
    }

    public void recordBufferCopy(long srcBuffer, long dstBuffer, long srcOffset, long dstOffset, long size) {
        Rosella.LOGGER.error("Recording buffer copy " + srcBuffer + "[" + srcOffset + "] -> " + dstBuffer + "[" + dstOffset + "] " + size);
        try (MemoryStack stack = MemoryStack.stackPush()) {
            VkBufferCopy.Buffer regions = VkBufferCopy.callocStack(1, stack);
            regions.get(0)
                .srcOffset(srcOffset)
                .dstOffset(dstOffset)
                .size(size);

            VK10.vkCmdCopyBuffer(this.commandBuffer, srcBuffer, dstBuffer, regions);
        }
    }

    public boolean hasAcquiredBuffer(long buffer) {
        return this.acquiredBuffers.contains(buffer);
    }

    public boolean hasReleasedBuffer(long buffer) {
        return this.releasedBuffers.contains(buffer);
    }

    public boolean hasReadBuffer(long buffer) {
        return this.bufferReads.contains(buffer);
    }

    public boolean hasWrittenBuffer(long buffer) {
        return this.bufferWrites.contains(buffer);
    }

    public void addTask(Task task) {
        if(task.shouldSignal()) {
            this.signalTasks.add(task);
        }
    }

    public void addReadBuffer(long buffer) {
        this.bufferReads.add(buffer);
    }

    public void addWriteBuffer(long buffer) {
        this.bufferWrites.add(buffer);
    }

    public void addAcquireTask(BufferAcquireTask task, boolean requiresBarrier) {
        this.acquireTasks.add(task);

        if(requiresBarrier) {
            this.acquiredBuffers.add(task.getBuffer());
        }
        if(task.shouldSignal()) {
            this.signalTasks.add(task);
        }
    }

    public void addReleaseTask(BufferReleaseTask task, boolean requiresBarrier) {
        this.releaseTasks.add(task);

        if(requiresBarrier) {
            this.releasedBuffers.add(task.getBuffer());
        }
        if(task.shouldSignal()) {
            this.signalTasks.add(task);
        }
    }

    public Set<Long> getWaitSemaphores() {
        return this.waitSemaphores;
    }

    public Set<Long> getSignalSemaphores() {
        return this.signalSemaphores;
    }

    public Set<Task> getSignalTasks() {
        return this.signalTasks;
    }

    public void reset() {
        signalTasks.clear();

        acquiredBuffers.clear();
        releasedBuffers.clear();
        bufferWrites.clear();
        bufferReads.clear();

        acquireTasks.clear();
        releaseTasks.clear();

        waitSemaphores.clear();
        signalSemaphores.clear();

        commandBuffer = null;
    }
}
