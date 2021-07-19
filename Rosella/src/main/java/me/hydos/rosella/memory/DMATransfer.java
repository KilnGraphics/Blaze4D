package me.hydos.rosella.memory;

import it.unimi.dsi.fastutil.longs.Long2ObjectOpenHashMap;
import it.unimi.dsi.fastutil.longs.LongOpenHashSet;
import it.unimi.dsi.fastutil.objects.ObjectOpenHashSet;
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
import java.nio.Buffer;
import java.nio.ByteBuffer;
import java.nio.IntBuffer;
import java.nio.LongBuffer;
import java.util.*;
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
        try {
            lock.lock();
            validateAcquireBuffer(buffer);

            this.ownedBuffers.add(buffer);
            if(waitSemaphores != null && !waitSemaphores.isEmpty()) {
                this.recordTask(new WaitSemaphoreTask(waitSemaphores));
            }
            if(srcQueue != this.transferQueue.getQueueFamily()) {
                this.recordTask(new PipelineBarrierTask(0, VK10.VK_PIPELINE_STAGE_TRANSFER_BIT)
                        .addBufferMemoryBarrier(
                                VK10.VK_ACCESS_MEMORY_WRITE_BIT | VK10.VK_ACCESS_MEMORY_READ_BIT,
                                VK10.VK_ACCESS_TRANSFER_WRITE_BIT | VK10.VK_ACCESS_TRANSFER_READ_BIT,
                                srcQueue, this.transferQueue.getQueueFamily(), buffer, 0, VK10.VK_WHOLE_SIZE));
            }
            if(completedCb != null) {
                this.recordTask(new CallbackTask(completedCb));
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
        try {
            lock.lock();
            validateAcquireBuffer(buffer);

            this.ownedBuffers.add(buffer);
            if(waitSemaphores != null && !waitSemaphores.isEmpty()) {
                this.recordTask(new WaitSemaphoreTask(waitSemaphores));
            }
            if(completedCb != null) {
                this.recordTask(new CallbackTask(completedCb));
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

            if(dstQueue != this.transferQueue.getQueueFamily()) {
                recordTask(new PipelineBarrierTask(VK10.VK_PIPELINE_STAGE_TRANSFER_BIT, 0)
                        .addBufferMemoryBarrier(
                                VK10.VK_ACCESS_TRANSFER_WRITE_BIT | VK10.VK_ACCESS_TRANSFER_READ_BIT,
                                VK10.VK_ACCESS_MEMORY_WRITE_BIT | VK10.VK_ACCESS_MEMORY_READ_BIT,
                                this.transferQueue.getQueueFamily(), dstQueue, buffer, 0, VK10.VK_WHOLE_SIZE));
            }
            if(signalSemaphores != null && !signalSemaphores.isEmpty()) {
                recordTask(new SignalSemaphoreTask(signalSemaphores));
            }
            if(completedCb != null) {
                recordTask(new CallbackTask(completedCb));
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
                this.recordTask(new SignalSemaphoreTask(signalSemaphores));
            }
            if(completedCb != null) {
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
     * @param srcBufferData The data to write into the destination buffer
     * @param dstBuffer The destination buffer
     * @param dstOffset The offset in the destination buffer to where the data should be copied to
     */
    public void transferBufferFromHost(ByteBuffer srcBufferData, long dstBuffer, long dstOffset) {
        final long size = srcBufferData.limit();
        try {
            lock.lock();
            if(!ownedBuffers.contains(dstBuffer)) {
                throw new RuntimeException("Cannot transfer to buffer that is not owned by the DMA engine!");
            }

            StagingMemoryPool.StagingMemoryAllocation staging = stagingMemory.allocate(size);
            MemoryUtil.memCopy(srcBufferData, staging.getHostBuffer());

            final long srcBuffer = staging.getVulkanBuffer();
            final long srcOffset = staging.getBufferOffset();


            PipelineBarrierTask barrier = new PipelineBarrierTask(
                    VK10.VK_PIPELINE_STAGE_HOST_BIT | VK10.VK_PIPELINE_STAGE_TRANSFER_BIT,
                    VK10.VK_PIPELINE_STAGE_TRANSFER_BIT
            );

            barrier.addBufferMemoryBarrier(VK10.VK_ACCESS_HOST_WRITE_BIT, VK10.VK_ACCESS_TRANSFER_READ_BIT,
                    0, 0,
                    srcBuffer, srcOffset, size);

            barrier.addBufferMemoryBarrier(VK10.VK_ACCESS_TRANSFER_READ_BIT | VK10.VK_ACCESS_TRANSFER_WRITE_BIT, VK10.VK_ACCESS_TRANSFER_WRITE_BIT,
                    0, 0,
                    dstBuffer, dstOffset, size);

            recordTask(barrier);
            recordTask(new BufferTransferTask(staging.getVulkanBuffer(), dstBuffer).addRegion(srcOffset, dstOffset, size));
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
     * @param dstBufferData The destination buffer
     * @param completedCb A function that is called once the transfer has completed
     */
    public void transferBufferToHost(long srcBuffer, long srcOffset, ByteBuffer dstBufferData, @Nullable Runnable completedCb) {
        final long size = dstBufferData.limit();
        try {
            lock.lock();
            if(!ownedBuffers.contains(srcBuffer)) {
                throw new RuntimeException("Cannot transfer from buffer that is now owned by the DMA engine!");
            }

            StagingMemoryPool.StagingMemoryAllocation staging = stagingMemory.allocate(size);

            final long dstBuffer = staging.getVulkanBuffer();
            final long dstOffset = staging.getBufferOffset();

            recordTask(new PipelineBarrierTask(VK10.VK_PIPELINE_STAGE_TRANSFER_BIT, VK10.VK_PIPELINE_STAGE_TRANSFER_BIT)
                    .addBufferMemoryBarrier(
                            VK10.VK_ACCESS_TRANSFER_WRITE_BIT, VK10.VK_ACCESS_TRANSFER_READ_BIT,
                            0, 0,
                            srcBuffer, srcOffset, size));

            recordTask(new BufferTransferTask(srcBuffer, staging.getVulkanBuffer()).addRegion(srcOffset, dstOffset, size));
            recordTask(new PipelineBarrierTask(VK10.VK_PIPELINE_STAGE_TRANSFER_BIT, VK10.VK_PIPELINE_STAGE_HOST_BIT)
                    .addBufferMemoryBarrier(
                            VK10.VK_ACCESS_TRANSFER_WRITE_BIT, VK10.VK_ACCESS_HOST_READ_BIT,
                            0, 0,
                            dstBuffer, dstOffset, size));
            recordTask(new CallbackTask(() -> {
                MemoryUtil.memCopy(staging.getHostBuffer(), dstBufferData);
                staging.free();
                if(completedCb != null) {
                    completedCb.run();
                }
            }));
        } finally {
            lock.unlock();
        }
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

            recordTask(new PipelineBarrierTask(VK10.VK_PIPELINE_STAGE_TRANSFER_BIT, VK10.VK_PIPELINE_STAGE_TRANSFER_BIT)
                    .addBufferMemoryBarrier(
                            VK10.VK_ACCESS_TRANSFER_WRITE_BIT, VK10.VK_ACCESS_TRANSFER_READ_BIT,
                            0, 0,
                            srcBuffer, srcOffset, size)
                    .addBufferMemoryBarrier(
                            VK10.VK_ACCESS_TRANSFER_WRITE_BIT | VK10.VK_ACCESS_TRANSFER_READ_BIT, VK10.VK_ACCESS_TRANSFER_WRITE_BIT,
                            0, 0,
                            dstBuffer, dstOffset, size));
            recordTask(new BufferTransferTask(srcBuffer, dstBuffer).addRegion(srcOffset, dstOffset, size));

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

    private class DMAWorker implements Runnable {

        private final int MAX_TASKS_PER_SUBMISSION = 40;

        private final VulkanQueue queue;

        private final long commandPool;
        private final VkCommandBuffer commandBuffer;
        private final long waitFence;

        private final DMARecorder recorder = new DMARecorder();

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

            this.recorder.reset();
            if(!currentTask.canRecord(this.recorder)) {
                return false;
            }

            // Record command buffers
            this.recorder.beginRecord(this.commandBuffer);
            while(currentTask != null && currentTask.canRecord(this.recorder)) {
                currentTask.record(this.recorder);
                try {
                    lock.lock();
                    nextTask = currentTask.getNext();
                    if(nextTask == null) {
                        lastTask = null;
                    }
                    currentTask = nextTask;
                } finally {
                    lock.unlock();
                }
            }
            this.recorder.endRecord();

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


                int result = this.queue.vkQueueSubmit(submitInfo, this.waitFence);
                if(result != VK10.VK_SUCCESS) {
                    throw new RuntimeException("Failed to submit transfer " + result);
                }

                result = VK10.vkWaitForFences(this.queue.getDevice(), this.waitFence, true, 1000 * 1000 * 10);
                if(result == VK10.VK_TIMEOUT) {
                    throw new RuntimeException("Transfer wait timed out");
                }
                if(result != VK10.VK_SUCCESS) {
                    throw new RuntimeException("Failed to wait for fence");
                }

                Rosella.LOGGER.error("Signaling callbacks");

                for(Runnable task : this.recorder.getSignalCallbacks()) {
                    task.run();
                }

                Rosella.LOGGER.error("Task completed");
            }

            return true;
        }
    }
}
