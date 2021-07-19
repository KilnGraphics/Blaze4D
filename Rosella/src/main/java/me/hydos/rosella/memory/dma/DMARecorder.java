package me.hydos.rosella.memory.dma;

import it.unimi.dsi.fastutil.longs.LongArraySet;
import it.unimi.dsi.fastutil.longs.LongOpenHashSet;
import it.unimi.dsi.fastutil.objects.ObjectOpenHashSet;
import me.hydos.rosella.Rosella;
import org.jetbrains.annotations.NotNull;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.vulkan.*;

import java.util.ArrayList;
import java.util.Collection;
import java.util.List;
import java.util.Set;

public class DMARecorder {

    private VkCommandBuffer commandBuffer;

    private final Set<Runnable> signalCallbacks = new ObjectOpenHashSet<>();

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
        }
    }

    public void endRecord() {
        int result = VK10.vkEndCommandBuffer(this.commandBuffer);
        if(result != VK10.VK_SUCCESS) {
            throw new RuntimeException("Failed to end transfer command buffer recording " + result);
        }
    }

    public void addWaitSemaphores(Collection<Long> semaphores) {
        this.waitSemaphores.addAll(semaphores);
    }

    public void addSignalSemaphores(Collection<Long> semaphores) {
        this.signalSemaphores.addAll(semaphores);
    }

    public void addCallback(Runnable callback) {
        this.signalCallbacks.add(callback);
    }

    public VkCommandBuffer getCommandBuffer() {
        return this.commandBuffer;
    }

    public Set<Long> getWaitSemaphores() {
        return this.waitSemaphores;
    }

    public Set<Long> getSignalSemaphores() {
        return this.signalSemaphores;
    }

    public Set<Runnable> getSignalCallbacks() {
        return this.signalCallbacks;
    }

    public void reset() {
        this.waitSemaphores.clear();
        this.signalSemaphores.clear();
        this.signalCallbacks.clear();

        commandBuffer = null;
    }
}
