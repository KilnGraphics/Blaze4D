package me.hydos.rosella.memory;

import it.unimi.dsi.fastutil.longs.Long2ObjectOpenHashMap;
import it.unimi.dsi.fastutil.longs.LongOpenHashSet;
import kotlin.NotImplementedError;
import me.hydos.rosella.memory.dma.BufferAcquireTask;
import me.hydos.rosella.memory.dma.BufferReleaseTask;
import me.hydos.rosella.memory.dma.DMARecorder;
import me.hydos.rosella.memory.dma.Task;
import org.jetbrains.annotations.Nullable;

import java.nio.ByteBuffer;
import java.util.Map;
import java.util.Set;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.locks.Condition;
import java.util.concurrent.locks.Lock;
import java.util.concurrent.locks.ReentrantLock;

public class DMATransfer {

    private enum ResourceState {
        ACQUIRE_QUEUED,
        ACQUIRED,
        RELEASE_QUEUED,
    }

    private final int transferQueueFamily = 0;

    private final Map<Long, ResourceState> ownedBuffers = new Long2ObjectOpenHashMap<>();
    private final Map<Long, ResourceState> ownedImages = new Long2ObjectOpenHashMap<>();

    private Task nextTask = null;
    private Task lastTask = null;

    private final Lock lock = new ReentrantLock();
    private final Condition taskAvailable = lock.newCondition();

    private final AtomicBoolean shouldTerminate = new AtomicBoolean(false);

    public DMATransfer() {
    }

    /**
     * Returns the queue family used for transfer operations. Callers should use this to properly release / acquire
     * their resources.
     *
     * @return The transfer queue family index
     */
    public int getTransferQueueFamily() {
        return this.transferQueueFamily;
    }

    /**
     * Performs a buffer acquire operation in the transfer queue making the buffer available to transfer operations.
     * The release operation from the source queue and any memory barriers <b>must</b> first be performed by the callee.
     * If the <code>srcQueue</code> is equals to the transfer queue <b>no memory barrier will be inserted</b> by the transfer engine.
     *
     * @param buffer The buffer to acquire
     * @param srcQueue The queue that previously had ownership of the buffer
     * @param waitSemaphores A list of semaphores to wait on before executing the acquire operation
     * @param completedCb A function that is called once the passed semaphores are safe to reuse
     */
    public void acquireBuffer(long buffer, int srcQueue, @Nullable long[] waitSemaphores, @Nullable Runnable completedCb) {
        try {
            lock.lock();
            if(this.ownedBuffers.containsKey(buffer)) {
                ResourceState currentState = this.ownedBuffers.get(buffer);
                if(currentState != ResourceState.RELEASE_QUEUED) {
                    throw new RuntimeException("Buffer is already owned by the DMA engine and no release operation is queued!");
                }
                if(waitSemaphores == null) {
                    throw new RuntimeException("Buffer has release operation queued but no wait semaphores are provided. Synchronization is required here!");
                }
            }

            if(srcQueue != this.transferQueueFamily || waitSemaphores != null) {
                this.recordTask(new BufferAcquireTask(true, buffer, srcQueue, this.transferQueueFamily, waitSemaphores, completedCb));
                this.ownedBuffers.put(buffer, ResourceState.ACQUIRE_QUEUED);

            } else {
                // No barrier is required
                this.ownedBuffers.put(buffer, ResourceState.ACQUIRED);

                if(completedCb != null) {
                    // TODO: make async
                    completedCb.run();
                }
            }

        } finally {
            lock.unlock();
        }
    }

    /**
     * Performs a image acquire operation in the transfer queue making the image available to transfer operations.
     * The release operation from the source queue and any memory barriers <b>must</b> first be performed by the callee.
     * If the <code>srcQueue</code> is equals to the transfer queue <b>no memory barrier will be inserted</b> by the transfer engine.
     *
     * @param image The image to acquire
     * @param srcQueue The queue that previously had ownership of the image
     * @param waitSemaphores A list of semaphores to wait on before executing the acquire operation
     * @param completedCb A function that is called once the passed semaphores are safe to reuse
     */
    public void acquireImage(long image, int srcQueue, @Nullable long[] waitSemaphores, @Nullable Runnable completedCb) {
        throw new NotImplementedError("Big F");
    }

    /**
     * Makes a shared buffer available to the transfer engine. This function does <b>not</b> perform any memory barrier
     * operations.
     *
     * @param buffer The buffer to acquire
     * @param waitSemaphores A list of semaphores to wait on before using the buffer
     * @param completedCb A function that is called once the passed semaphores are safe to reuse
     */
    public void acquireSharedBuffer(long buffer, @Nullable long[] waitSemaphores, @Nullable Runnable completedCb) {
        try {
            lock.lock();
            if(this.ownedBuffers.containsKey(buffer)) {
                ResourceState currentState = this.ownedBuffers.get(buffer);
                if(currentState != ResourceState.RELEASE_QUEUED) {
                    throw new RuntimeException("Buffer is already owned by the DMA engine and no release operation is queued!");
                }
                if(waitSemaphores == null) {
                    throw new RuntimeException("Buffer has release operation queued but no wait semaphores are provided. Synchronization is required here!");
                }
            }

            if(waitSemaphores != null) {
                this.recordTask(new BufferAcquireTask(true, buffer, 0, 0, waitSemaphores, completedCb));
                this.ownedBuffers.put(buffer, ResourceState.ACQUIRE_QUEUED);

            } else {
                // No barrier task is required
                this.ownedBuffers.put(buffer, ResourceState.ACQUIRED);

                if(completedCb != null) {
                    // TODO: make async
                    completedCb.run();
                }
            }

        } finally {
            lock.unlock();
        }
    }

    /**
     * Makes a shared image available to the transfer engine. This function does <b>not</b> perform any memory barrier
     * operations.
     *
     * @param image The image to acquire
     * @param waitSemaphores A list of semaphores to wait on before using the image
     * @param completedCb A function that is called once the passed semaphores are safe to reuse
     */
    public void acquireSharedImage(long image, @Nullable long[] waitSemaphores, @Nullable Runnable completedCb) {
        throw new NotImplementedError("Big F");
    }

    /**
     * Performs a buffer release operation on the transfer queue.
     * The acquire operation in the destination queue must be performed by the callee afterwards.
     * If the <code>dstQueue</code> is equals to the transfer queue <b>no memory barrier will be inserted</b> by the transfer engine.
     *
     * @param buffer The buffer to release
     * @param dstQueue The queue that will next take ownership of the buffer
     * @param signalSemaphores A list of semaphores to signal when the operation is complete
     * @param completedCb A function that is called once the release operation has completed
     */
    public void releaseBuffer(long buffer, int dstQueue, @Nullable long[] signalSemaphores, @Nullable Runnable completedCb) {
        try {
            lock.lock();
            if(this.ownedBuffers.getOrDefault(buffer, ResourceState.RELEASE_QUEUED) == ResourceState.RELEASE_QUEUED) {
                throw new RuntimeException("Buffer already has a release operation queued or buffer is not owned by the DMA engine!");
            }

            if(dstQueue != this.transferQueueFamily || signalSemaphores != null) {
                this.recordTask(new BufferReleaseTask(true, buffer, this.transferQueueFamily, dstQueue, signalSemaphores, completedCb));
                this.ownedBuffers.put(buffer, ResourceState.RELEASE_QUEUED);

            } else {
                // No barrier is required
                this.ownedBuffers.remove(buffer);

                if(completedCb != null) {
                    // TODO: make async
                    completedCb.run();
                }
            }

        } finally {
            lock.unlock();
        }
    }

    /**
     * Performs a image release operation on the transfer queue.
     * The acquire operation in the destination queue must be performed by the callee afterwards.
     * If the <code>dstQueue</code> is equals to the transfer queue <b>no memory barrier will be inserted</b> by the transfer engine.
     *
     * @param image The image to release
     * @param dstQueue The queue that will next take ownership of the image
     * @param signalSemaphores A list of semaphores to signal when the operation is complete
     * @param completedCb A function that is called once the release operation has completed
     */
    public void releaseImage(long image, int dstQueue, @Nullable long[] signalSemaphores, @Nullable Runnable completedCb) {
        throw new NotImplementedError("Big F");
    }

    /**
     * Removes a shared buffer from the available buffers in the transfer engine. This function does <b>not</b> perform
     * any memory barrier operations.
     *
     * @param buffer The buffer to release
     * @param signalSemaphores A list of semaphores to signal when the buffer is ready to use
     * @param completedCb A function that is called once the release operation has completed
     */
    public void releaseSharedBuffer(long buffer, @Nullable long[] signalSemaphores, @Nullable Runnable completedCb) {
        try {
            lock.lock();
            if(this.ownedBuffers.getOrDefault(buffer, ResourceState.RELEASE_QUEUED) == ResourceState.RELEASE_QUEUED) {
                throw new RuntimeException("Buffer already has a release operation queued or buffer is not owned by the DMA engine!");
            }

            if(signalSemaphores != null) {
                this.recordTask(new BufferReleaseTask(true, buffer, 0, 0, signalSemaphores, completedCb));
                this.ownedBuffers.put(buffer, ResourceState.RELEASE_QUEUED);

            } else {
                // No barrier is required
                this.ownedBuffers.remove(buffer);

                if(completedCb != null) {
                    // TODO: make async
                    completedCb.run();
                }
            }

        } finally {
            lock.unlock();
        }
    }

    /**
     * Removes a shared image from the available images in the transfer engine. This function does <b>not</b> perform
     * any memory barrier operations.
     *
     * @param image The image to release
     * @param signalSemaphores A list of semaphores to signal when the image is ready to use
     * @param completedCb A function that is called once the release operation has completed
     */
    public void releaseSharedImage(long image, @Nullable long[] signalSemaphores, @Nullable Runnable completedCb) {
        throw new NotImplementedError("Big F");
    }

    /**
     * Performs a transfer operation from host memory to a buffer. The entire source buffer range will be copied into
     * the destination buffer at the specified offset.
     * The destination buffer must first be made available to the transfer engine by calling any of the acquire functions.
     * The source buffer will be copied and can safely be overwritten after this function returns.
     *
     * @param srcBuffer The data to write into the destination buffer
     * @param dstBuffer The destination buffer
     * @param dstOffset The offset in the destination buffer to where the data should be copied to
     */
    public void transferBufferFromHost(ByteBuffer srcBuffer, long dstBuffer, long dstOffset) {
        throw new NotImplementedError("Big F");
    }

    /**
     * Performs a transfer operation from a buffer to host memory. A region of the same length as the destination
     * buffer will be copied from the source buffer at the specified offset into the destination buffer.
     * The source buffer must first be made available to the transfer engine by calling any of the acquire functions.
     * The transfer engine will take ownership of the destination buffer until the operation is complete. The
     * destination buffer <b>must not</b> be modified until the operation has completed.
     *
     * @param srcBuffer The source buffer
     * @param srcOffset The offset in the source buffer from where the data should be copied from
     * @param dstBuffer The destination buffer
     * @param completedCb A function that is called once the transfer has completed
     */
    public void transferBufferToHost(long srcBuffer, long srcOffset, ByteBuffer dstBuffer, @Nullable Runnable completedCb) {
        throw new NotImplementedError("Big F");
    }


    // TODO: Images are pain
    public void transferImageFromHost(ByteBuffer srcBuffer, long dstImage, int dstImageLayout, BufferImageCopy copy) {
        throw new NotImplementedError("Big F");
    }

    public void transferImageToHost(long srcImage, int srcImageLayout, ByteBuffer dstBuffer, BufferImageCopy copy) {
        throw new NotImplementedError("Big F");
    }

    /**
     * Performs a transfer operation between 2 buffers.
     * Both buffers must first be made available to the transfer engine by calling any of the acquire functions.
     *
     * @param srcBuffer The source buffer
     * @param srcOffset The offset in the source buffer
     * @param dstBuffer The destination buffer
     * @param dstOffset The offset in the destination buffer
     * @param size The amount of data to copy
     */
    public void transferBuffer(long srcBuffer, long srcOffset, long dstBuffer, long dstOffset, long size) {
        throw new NotImplementedError("Big F");
    }

    private void recordTask(Task task) {
        if(lastTask == null) {
            nextTask = task;
        } else {
            lastTask.setNext(task);
        }
        lastTask = task;
        taskAvailable.signal();
    }

    public record BufferImageCopy(long bufferOffset, int bufferRowLength, int bufferImageHeight) { // TODO: pain
    }

    private class DMAWorker implements Runnable {

        DMARecorder recorder = new DMARecorder();

        @Override
        public void run() {
            while(!shouldTerminate.get()) {
                if(!tryRunTask()) {
                    try {
                        lock.lock();
                        taskAvailable.awaitNanos(1000);
                    } catch (InterruptedException ignored) {
                        // TODO: ???
                    } finally {
                        lock.unlock();
                    }
                }
            }
        }

        private boolean tryRunTask() {
            Task currentTask;
            try {
                lock.lock();
                if(nextTask == null) {
                    return false;
                }
                currentTask = nextTask;
            } finally {
                lock.unlock();
            }

            recorder.reset();
            recorder.begin();
            for(int taskIndex = 0; (taskIndex < 20) && (currentTask != null); taskIndex++) { // TODO: Max tasks constant
                if(!currentTask.canRecord(recorder)) {
                    try {
                        lock.lock();
                        nextTask = currentTask;
                    } finally {
                        lock.unlock();
                    }
                    break;
                }

                currentTask.record(recorder);

                try {
                    lock.lock();
                    currentTask = currentTask.getNext();
                    if(currentTask == null) {
                        nextTask = null;
                        lastTask = null;
                    }
                } finally {
                    lock.unlock();
                }
            }
            recorder.end();

            return true;
        }
    }
}
