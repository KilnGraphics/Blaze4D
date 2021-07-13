package me.hydos.rosella.memory.dma;

import it.unimi.dsi.fastutil.longs.LongArraySet;
import org.jetbrains.annotations.Nullable;
import org.lwjgl.vulkan.VK10;
import org.lwjgl.vulkan.VkBufferMemoryBarrier;

import java.util.Collection;
import java.util.Set;

public class BufferAcquireTask extends Task {

    private final Runnable completeCb;
    private final Set<Long> waitSemaphores;

    private final long buffer;
    private final int srcQueue;
    private final int dstQueue;

    public BufferAcquireTask(long buffer, int srcQueue, int dstQueue, @Nullable Collection<Long> waitSemaphores, @Nullable Runnable completeCb) {
        super();

        this.completeCb = completeCb;
        this.waitSemaphores = new LongArraySet();
        if(waitSemaphores != null) {
            this.waitSemaphores.addAll(waitSemaphores);
        }

        this.buffer = buffer;
        this.srcQueue = srcQueue;
        this.dstQueue = dstQueue;
    }

    @Override
    public boolean canRecord(DMARecorder recorder) {
        return false;
    }

    @Override
    public void record(DMARecorder recorder) {
    }

    @Override
    public boolean shouldSignal() {
        return true;
    }

    @Override
    public void onCompleted() {
        if(completeCb != null) {
            // TODO: make async
            completeCb.run();
        }
    }

    public long getBuffer() {
        return buffer;
    }

    public boolean isBarrierRequired() {
        return this.srcQueue == this.dstQueue;
    }

    public Set<Long> getWaitSemaphores() {
        return waitSemaphores;
    }

    public void fillBufferBarrier(VkBufferMemoryBarrier barrier) {
        barrier.sType(VK10.VK_STRUCTURE_TYPE_BUFFER_MEMORY_BARRIER);
        barrier.pNext(0);
        barrier.srcAccessMask(VK10.VK_ACCESS_MEMORY_READ_BIT | VK10.VK_ACCESS_MEMORY_WRITE_BIT);
        barrier.dstAccessMask(VK10.VK_ACCESS_TRANSFER_READ_BIT | VK10.VK_ACCESS_TRANSFER_WRITE_BIT);
        barrier.srcQueueFamilyIndex(this.srcQueue);
        barrier.dstQueueFamilyIndex(this.dstQueue);
        barrier.buffer(this.buffer);
        barrier.offset(0);
        barrier.size(VK10.VK_WHOLE_SIZE);
    }
}
