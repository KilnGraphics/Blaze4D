package me.hydos.rosella.memory.dma;

import it.unimi.dsi.fastutil.longs.LongArraySet;
import it.unimi.dsi.fastutil.longs.LongOpenHashSet;
import it.unimi.dsi.fastutil.objects.ObjectOpenHashSet;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.device.VulkanQueue;
import org.apache.logging.log4j.core.config.composite.MergeStrategy;
import org.jetbrains.annotations.NotNull;
import org.lwjgl.PointerBuffer;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.vulkan.*;

import java.nio.IntBuffer;
import java.nio.LongBuffer;
import java.util.*;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;

public class DMARecorder {

    private final VkCommandBuffer commandBuffer;
    private final VulkanQueue queue;
    private final long completedFence;

    private final Deque<Runnable> signalCallbacks = new ArrayDeque<>();

    private final Set<Long> waitSemaphores = new LongArraySet();
    private final Set<Long> signalSemaphores = new LongArraySet();

    private final AtomicInteger state = new AtomicInteger(0);
    private final Thread waiterThread;

    public DMARecorder(VkCommandBuffer commandBuffer, VulkanQueue queue) {
        this.commandBuffer = commandBuffer;
        this.queue = queue;

        try(MemoryStack stack = MemoryStack.stackPush()) {
            VkFenceCreateInfo fenceInfo = VkFenceCreateInfo.callocStack(stack);
            fenceInfo.sType(VK10.VK_STRUCTURE_TYPE_FENCE_CREATE_INFO);

            LongBuffer pFence = stack.mallocLong(1);
            int result = VK10.vkCreateFence(queue.getDevice(), fenceInfo, null, pFence);
            if (result != VK10.VK_SUCCESS) {
                throw new RuntimeException("Failed to create wait fence " + result);
            }
            this.completedFence = pFence.get();
        }

        this.waiterThread = new Thread(this::runWaiter);
        this.waiterThread.start();
    }

    public void destroy() {
        this.state.set(-1); // TODO: how to free the fence?
    }

    public boolean isReady() {
        return state.get() == 0;
    }

    public void beginRecord() {
        int result = VK10.vkResetCommandBuffer(this.commandBuffer, 0);
        if(result != VK10.VK_SUCCESS) {
            throw new RuntimeException("Failed to reset command buffer " + result);
        }

        try (MemoryStack stack = MemoryStack.stackPush()) {
            VkCommandBufferBeginInfo beginInfo = VkCommandBufferBeginInfo.callocStack(stack);
            beginInfo.sType(VK10.VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO);
            beginInfo.flags(VK10.VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT);

            result = VK10.vkBeginCommandBuffer(this.commandBuffer, beginInfo);
            if(result != VK10.VK_SUCCESS) {
                throw new RuntimeException("Failed to begin recoding of transfer command buffer " + result);
            }
        }
    }

    public void endRecord() {
        int result = VK10.vkEndCommandBuffer(this.commandBuffer);
        if(result != VK10.VK_SUCCESS) {
            throw new RuntimeException("Failed to end transfer command buffer recording " + result);
        }
    }

    public void submit() {
        state.set(1);

        try(MemoryStack stack = MemoryStack.stackPush()) {
            LongBuffer pWaitSem = null;
            IntBuffer pWaitSemStage = null;
            if(this.waitSemaphores.size() != 0) {
                pWaitSem = stack.mallocLong(this.waitSemaphores.size());
                pWaitSemStage = stack.mallocInt(this.waitSemaphores.size());
                for(long semaphore : this.waitSemaphores) {
                    pWaitSem.put(semaphore);
                    pWaitSemStage.put(VK10.VK_PIPELINE_STAGE_TRANSFER_BIT);
                }
                pWaitSem.rewind();
            }

            LongBuffer pSignalSem = null;
            if(this.signalSemaphores.size() != 0) {
                pSignalSem = stack.mallocLong(this.signalSemaphores.size());
                for(long semaphore : this.signalSemaphores) {
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

            int result = this.queue.vkQueueSubmit(submitInfo, this.completedFence);
            if(result != VK10.VK_SUCCESS) {
                throw new RuntimeException("Failed to submit transfer " + result);
            }

            this.waitSemaphores.clear();
            this.signalSemaphores.clear();
        }
    }

    public VkCommandBuffer getCommandBuffer() {
        return this.commandBuffer;
    }

    public void addWaitSemaphores(Collection<Long> semaphores) {
        this.waitSemaphores.addAll(semaphores);
    }

    public void addSignalSemaphores(Collection<Long> semaphores) {
        this.signalSemaphores.addAll(semaphores);
    }

    public void addCallback(Runnable callback) {
        this.signalCallbacks.addLast(callback);
    }

    public boolean containsSignalSemaphores() {
        return !this.signalSemaphores.isEmpty();
    }

    private void runWaiter() {
        int result;
        while(true) {
            do {
                result = VK10.vkWaitForFences(this.queue.getDevice(), this.completedFence, true, 1000 * 1000);
                if(this.state.get() == -1) {
                    return;
                }
            } while (result == VK10.VK_TIMEOUT);

            if (result != VK10.VK_SUCCESS) {
                Rosella.LOGGER.fatal("Failed to wait for fences in DMARecorder waiting thread");
                // TODO kill everything
                return;
            }

            while (!this.signalCallbacks.isEmpty()) {
                Runnable next = this.signalCallbacks.pollFirst();
                try {
                    next.run();
                } catch (Exception ex) {
                    Rosella.LOGGER.error("Exception in DMARecorder callback", ex);
                }
            }

            result = VK10.vkResetFences(this.queue.getDevice(), this.completedFence);
            if(result != VK10.VK_SUCCESS) {
                Rosella.LOGGER.fatal("Failed to reset fence in DMARecorder waiting thread");
                // TODO kill everything
                return;
            }

            this.state.set(0);
        }
    }
}
