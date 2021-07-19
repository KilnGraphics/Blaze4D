package me.hydos.rosella.memory.dma;

import it.unimi.dsi.fastutil.longs.LongArraySet;
import me.hydos.rosella.Rosella;
import org.jetbrains.annotations.Nullable;
import org.lwjgl.vulkan.VK10;
import org.lwjgl.vulkan.VkBufferMemoryBarrier;

import java.util.Set;

public class BufferReleaseTask extends Task implements BufferBarrierTask {

    private final Runnable completeCb;
    private final Set<Long> signalSemaphores;

    private final long buffer;
    private final int srcQueue;
    private final int dstQueue;

    public BufferReleaseTask(long buffer, int srcQueue, int dstQueue, @Nullable Set<Long> signalSemaphores, @Nullable Runnable completedCb) {
        super();

        this.completeCb = completedCb;
        this.signalSemaphores = new LongArraySet();
        if(signalSemaphores != null) {
            this.signalSemaphores.addAll(signalSemaphores);
        }

        this.buffer = buffer;
        this.srcQueue = srcQueue;
        this.dstQueue = dstQueue;
    }

    @Override
    public boolean scan(DMARecorder recorder) {
        if(recorder.hasReleasedBuffer(this.buffer)) {
            return false;
        }

        recorder.addReleaseTask(this, this.srcQueue != this.dstQueue);
        return true;
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
        if(this.completeCb != null) {
            // TODO: make async
            this.completeCb.run();
        }
    }

    public long getBuffer() {
        return buffer;
    }

    @Override
    public void fillBarrier(VkBufferMemoryBarrier barrier) {
        barrier.sType(VK10.VK_STRUCTURE_TYPE_BUFFER_MEMORY_BARRIER);
        barrier.pNext(0);
        barrier.srcAccessMask(VK10.VK_ACCESS_TRANSFER_READ_BIT | VK10.VK_ACCESS_TRANSFER_WRITE_BIT);
        barrier.dstAccessMask(VK10.VK_ACCESS_MEMORY_READ_BIT | VK10.VK_ACCESS_MEMORY_WRITE_BIT);
        barrier.srcQueueFamilyIndex(this.srcQueue);
        barrier.dstQueueFamilyIndex(this.dstQueue);
        barrier.buffer(buffer);
        barrier.offset(0);
        barrier.size(VK10.VK_WHOLE_SIZE);
    }
}
