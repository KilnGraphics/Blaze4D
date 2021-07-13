package me.hydos.rosella.memory.dma;

import org.jetbrains.annotations.Nullable;
import org.lwjgl.vulkan.VK10;
import org.lwjgl.vulkan.VkBufferMemoryBarrier;

public class BufferAcquireTask extends Task {

    private final Runnable completeCb;
    private final long[] waitSemaphores;

    private final long buffer;
    private final int srcQueue;
    private final int dstQueue;

    public BufferAcquireTask(boolean initialReady, long buffer, int srcQueue, int dstQueue, @Nullable long[] waitSemaphores, @Nullable Runnable completeCb) {
        super(initialReady);

        this.completeCb = completeCb;
        this.waitSemaphores = waitSemaphores;

        this.buffer = buffer;
        this.srcQueue = srcQueue;
        this.dstQueue = dstQueue;
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

    public long[] getWaitSemaphores() {
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
