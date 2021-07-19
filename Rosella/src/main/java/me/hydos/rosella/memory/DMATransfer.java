package me.hydos.rosella.memory;

import it.unimi.dsi.fastutil.longs.Long2ObjectOpenHashMap;
import it.unimi.dsi.fastutil.longs.LongOpenHashSet;
import kotlin.NotImplementedError;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.device.VulkanQueue;
import me.hydos.rosella.memory.allocators.HostMappedAllocation;
import me.hydos.rosella.memory.dma.*;
import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;
import org.lwjgl.PointerBuffer;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.system.MemoryUtil;
import org.lwjgl.vulkan.*;

import javax.sql.rowset.RowSetWarning;
import java.nio.ByteBuffer;
import java.nio.IntBuffer;
import java.nio.LongBuffer;
import java.util.Deque;
import java.util.LinkedList;
import java.util.Map;
import java.util.Set;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.locks.Condition;
import java.util.concurrent.locks.Lock;
import java.util.concurrent.locks.ReentrantLock;

public class DMATransfer {

    private final VulkanQueue transferQueue;

    // These 2 keep the state after all queued transfers have completed. i.e. the state that is relevant for queueing up more instructions
    private final Set<Long> ownedBuffers = new LongOpenHashSet();
    private final Set<Long> ownedImages = new LongOpenHashSet();

    private Task nextTask = null;
    private Task lastTask = null;

    private final StagingMemoryPool stagingMemory;

    private final Lock lock = new ReentrantLock();
    private final Condition taskAvailable = lock.newCondition();

    private final AtomicBoolean shouldTerminate = new AtomicBoolean(false);

    private Thread worker;

    public DMATransfer(VulkanQueue transferQueue, long vmaAllocator) {
        this.transferQueue = transferQueue;
        this.worker = new Thread(new DMAWorker(this.transferQueue));
        this.worker.start();
        this.stagingMemory = new StagingMemoryPool(vmaAllocator);
    }

    /**
     * Returns the queue family used for transfer operations. Callers should use this to properly release / acquire
     * their resources.
     *
     * @return The transfer queue family index
     */
    public int getTransferQueueFamily() {
        return this.transferQueue.getQueueFamily();
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
    public void acquireBuffer(long buffer, int srcQueue, @Nullable Set<Long> waitSemaphores, @Nullable Runnable completedCb) {
        boolean runCompleted = false;
        try {
            lock.lock();
            validateAcquireBuffer(buffer);

            this.ownedBuffers.add(buffer);
            if(srcQueue != this.transferQueue.getQueueFamily() || waitSemaphores != null) {
                this.recordTask(new BufferAcquireTask(buffer, srcQueue, this.transferQueue.getQueueFamily(), waitSemaphores, completedCb));

            } else {
                // No acquire operation is required
                runCompleted = completedCb != null;
            }

        } finally {
            lock.unlock();
        }

        if(runCompleted) {
            completedCb.run();
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
    public void acquireImage(long image, int srcQueue, @Nullable Set<Long> waitSemaphores, @Nullable Runnable completedCb) {
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
    public void acquireSharedBuffer(long buffer, @Nullable Set<Long> waitSemaphores, @Nullable Runnable completedCb) {
        boolean runCompleted = false;
        try {
            lock.lock();
            validateAcquireBuffer(buffer);

            this.ownedBuffers.add(buffer);
            if(waitSemaphores != null) {
                this.recordTask(new BufferAcquireTask(buffer, 0, 0, waitSemaphores, completedCb));

            } else {
                // No acquire operation is required
                runCompleted = completedCb != null;
            }

        } finally {
            lock.unlock();
        }

        if(runCompleted) {
            completedCb.run();
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
    public void acquireSharedImage(long image, @Nullable Set<Long> waitSemaphores, @Nullable Runnable completedCb) {
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
    public void releaseBuffer(long buffer, int dstQueue, @Nullable Set<Long> signalSemaphores, @Nullable Runnable completedCb) {
        try {
            lock.lock();
            validateReleaseBuffer(buffer);

            if(dstQueue != this.transferQueue.getQueueFamily() || signalSemaphores != null) {
                this.recordTask(new BufferReleaseTask(buffer, this.transferQueue.getQueueFamily(), dstQueue, signalSemaphores, completedCb));

            } else {
                // No release operation is required
                this.recordTask(new CallbackTask(completedCb));
            }
            this.ownedBuffers.remove(buffer);

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
    public void releaseImage(long image, int dstQueue, @Nullable Set<Long> signalSemaphores, @Nullable Runnable completedCb) {
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
    public void releaseSharedBuffer(long buffer, @Nullable Set<Long> signalSemaphores, @Nullable Runnable completedCb) {
        try {
            lock.lock();

            if(signalSemaphores != null) {
                this.recordTask(new BufferReleaseTask(buffer, 0, 0, signalSemaphores, completedCb));

            } else {
                // No release operation is required
                this.recordTask(new CallbackTask(completedCb));
            }
            this.ownedBuffers.remove(buffer);

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
    public void releaseSharedImage(long image, @Nullable Set<Long> signalSemaphores, @Nullable Runnable completedCb) {
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
        try {
            lock.lock();
            if(!ownedBuffers.contains(dstBuffer)) {
                throw new RuntimeException("Cannot transfer to buffer that is not owned by the DMA engine!");
            }

            StagingMemoryPool.StagingMemoryAllocation staging = stagingMemory.allocate(srcBuffer.limit());
            MemoryUtil.memCopy(srcBuffer, staging.getHostBuffer());

            recordTask(new BufferTransferTask(staging.getVulkanBuffer(), dstBuffer, staging.getBufferOffset(), dstOffset, srcBuffer.limit()));
            recordTask(new CallbackTask(staging::free));
        } finally {
            lock.unlock();
        }
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
        try {
            lock.lock();
            if(!ownedBuffers.contains(srcBuffer)) {
                throw new RuntimeException("Cannot transfer from buffer that is now owned by the DMA engine!");
            }

            StagingMemoryPool.StagingMemoryAllocation staging = stagingMemory.allocate(dstBuffer.limit());

            recordTask(new BufferTransferTask(srcBuffer, staging.getVulkanBuffer(), srcOffset, staging.getBufferOffset(), dstBuffer.limit()));
            recordTask(new CallbackTask(() -> {
                MemoryUtil.memCopy(staging.getHostBuffer(), dstBuffer);
                staging.free();
                if(completedCb != null) {
                    completedCb.run();
                }
            }));
        } finally {
            lock.unlock();
        }
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
        try {
            lock.lock();
            if (!ownedBuffers.contains(srcBuffer)) {
                throw new RuntimeException("Cannot transfer from a buffer that is not owned by the DMA engine!");
            }
            if (!ownedBuffers.contains(dstBuffer)) {
                throw new RuntimeException("Cannot transfer to a buffer that is not owned by the DMA engine!");
            }

            recordTask(new BufferTransferTask(srcBuffer, dstBuffer, srcOffset, dstOffset, size));

        } finally {
            lock.unlock();
        }
    }

    private void validateAcquireBuffer(long buffer) {
        if(this.ownedBuffers.contains(buffer)) {
            throw new RuntimeException("Cannot acquire buffer that is already owned by the DMA engine");
        }
    }

    private void validateReleaseBuffer(long buffer) {
        if(!this.ownedBuffers.contains(buffer)) {
            throw new RuntimeException("Cannot release buffer that is not owned by the DMA engine");
        }
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

        private final int MAX_TASKS_PER_SUBMISSION = 40;

        private final VulkanQueue queue;

        private final long commandPool;
        private final VkCommandBuffer commandBuffer;
        private final long waitFence;

        private final DMARecorder recorder = new DMARecorder();
        private final Deque<Task> currentTasks = new LinkedList<>();

        public DMAWorker(@NotNull VulkanQueue queue) {
            this.queue = queue;

            try (MemoryStack stack = MemoryStack.stackPush()) {
                VkCommandPoolCreateInfo createInfo = VkCommandPoolCreateInfo.callocStack(stack);
                createInfo.sType(VK10.VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO);
                createInfo.flags(VK10.VK_COMMAND_POOL_CREATE_TRANSIENT_BIT | VK10.VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT);
                createInfo.queueFamilyIndex(queue.getQueueFamily());

                LongBuffer pPool = stack.longs(0);

                int result = VK10.vkCreateCommandPool(queue.getDevice(), createInfo, null, pPool);
                if(result != VK10.VK_SUCCESS) {
                    throw new RuntimeException("Failed to create command pool for DMAWorker " + result);
                }

                this.commandPool = pPool.get();

                VkCommandBufferAllocateInfo allocInfo = VkCommandBufferAllocateInfo.callocStack(stack);
                allocInfo.sType(VK10.VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO);
                allocInfo.commandPool(this.commandPool);
                allocInfo.level(VK10.VK_COMMAND_BUFFER_LEVEL_PRIMARY);
                allocInfo.commandBufferCount(1);

                PointerBuffer pBuffers = stack.pointers(0);
                result = VK10.vkAllocateCommandBuffers(queue.getDevice(), allocInfo, pBuffers);
                if(result != VK10.VK_SUCCESS) {
                    throw new RuntimeException("Failed to allocate command buffers for DMAWorker " + result);
                }

                this.commandBuffer = new VkCommandBuffer(pBuffers.get(), queue.getDevice());

                VkFenceCreateInfo fenceInfo = VkFenceCreateInfo.callocStack(stack);
                fenceInfo.sType(VK10.VK_STRUCTURE_TYPE_FENCE_CREATE_INFO);

                result = VK10.vkCreateFence(queue.getDevice(), fenceInfo, null, pPool.rewind());
                if(result != VK10.VK_SUCCESS) {
                    throw new RuntimeException("Failed to create wait fence " + result);
                }
                this.waitFence = pPool.get();
            }
        }

        @Override
        public void run() {
            while(!shouldTerminate.get()) {
                if(!tryRunTask()) {
                    try {
                        lock.lock();
                        taskAvailable.awaitNanos(1000000L);
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

            Rosella.LOGGER.error("Found runnable Task");

            this.recorder.reset();
            this.currentTasks.clear();

            // Build list of tasks that should be executed in this pass
            for(int taskIndex = 0; taskIndex < MAX_TASKS_PER_SUBMISSION; taskIndex++) {
                if(!currentTask.scan(this.recorder)) {
                    break;
                }

                this.currentTasks.addLast(currentTask);

                try {
                    lock.lock();
                    nextTask = currentTask.getNext();
                    if(nextTask == null) {
                        lastTask = null;
                        break;
                    }
                    currentTask = nextTask;
                } finally {
                    lock.unlock();
                }
            }

            if(currentTasks.isEmpty()) {
                return false;
            }

            Rosella.LOGGER.error("Task has been built");

            // Record command buffers
            this.recorder.beginRecord(this.commandBuffer);
            while(!currentTasks.isEmpty()) {
                currentTasks.pollFirst().record(this.recorder);
            }
            this.recorder.endRecord();

            Rosella.LOGGER.error("Task recording completed");

            // Submit commands
            try (MemoryStack stack = MemoryStack.stackPush()) {
                Set<Long> waitSemaphores = this.recorder.getWaitSemaphores();
                Set<Long> signalSemaphores = this.recorder.getSignalSemaphores();

                LongBuffer pWaitSem = null;
                IntBuffer pWaitSemStage = null;
                if(waitSemaphores.size() != 0) {
                    pWaitSem = stack.mallocLong(waitSemaphores.size());
                    pWaitSemStage = stack.mallocInt(waitSemaphores.size());
                    for(long semaphore : waitSemaphores) {
                        pWaitSem.put(semaphore);
                        pWaitSemStage.put(VK10.VK_PIPELINE_STAGE_TRANSFER_BIT);
                    }
                    pWaitSem.rewind();
                }

                LongBuffer pSignalSem = null;
                if(signalSemaphores.size() != 0) {
                    pSignalSem = stack.mallocLong(signalSemaphores.size());
                    for(long semaphore : signalSemaphores) {
                        pSignalSem.put(semaphore);
                    }
                    pSignalSem.rewind();
                }

                PointerBuffer pCmdBuffer = stack.pointers(this.commandBuffer);

                VkSubmitInfo submitInfo = VkSubmitInfo.callocStack(stack);
                submitInfo.sType(VK10.VK_STRUCTURE_TYPE_SUBMIT_INFO);
                submitInfo.pWaitSemaphores(pWaitSem);
                submitInfo.pWaitDstStageMask(pWaitSemStage);
                submitInfo.pCommandBuffers(pCmdBuffer);
                submitInfo.pSignalSemaphores(pSignalSem);

                Rosella.LOGGER.error("Submitting task");

                int result = this.queue.vkQueueSubmit(submitInfo, this.waitFence);
                if(result != VK10.VK_SUCCESS) {
                    throw new RuntimeException("Failed to submit transfer " + result);
                }

                Rosella.LOGGER.error("Waiting for completion");

                result = VK10.vkWaitForFences(this.queue.getDevice(), this.waitFence, true, 1000 * 1000 * 10);
                if(result == VK10.VK_TIMEOUT) {
                    throw new RuntimeException("Transfer wait timed out");
                }
                if(result != VK10.VK_SUCCESS) {
                    throw new RuntimeException("Failed to wait for fence");
                }

                Rosella.LOGGER.error("Signaling callbacks");

                for(Task task : this.recorder.getSignalTasks()) {
                    task.onCompleted();
                }

                Rosella.LOGGER.error("Task completed");
            }

            return true;
        }
    }
}
