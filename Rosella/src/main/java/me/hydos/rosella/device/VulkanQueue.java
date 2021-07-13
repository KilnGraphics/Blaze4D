package me.hydos.rosella.device;

import org.lwjgl.vulkan.*;

import java.util.concurrent.locks.Lock;
import java.util.concurrent.locks.ReentrantLock;

public final class VulkanQueue {

    private final VkQueue queue;
    private final int family;

    private final Lock lock = new ReentrantLock();

    public VulkanQueue(VkQueue queue, int family) {
        this.queue = queue;
        this.family = family;
    }

    public VkQueue getQueue() {
        return this.queue;
    }

    public VkDevice getDevice() {
        return this.queue.getDevice();
    }

    public int getQueueFamily() {
        return this.family;
    }

    public Lock getLock() {
        return lock;
    }

    public int vkQueueSubmit(VkSubmitInfo submit, long fence) {
        int result;
        try {
            lock.lock();
            result = VK10.vkQueueSubmit(this.queue, submit, fence);
        } finally {
            lock.unlock();
        }
        return result;
    }

    public int vkQueueSubmit(VkSubmitInfo.Buffer pSubmits, long fence) {
        int result;
        try {
            lock.lock();
            result = VK10.vkQueueSubmit(this.queue, pSubmits, fence);
        } finally {
            lock.unlock();
        }
        return result;
    }

    public int vkQueueBindSparse(VkBindSparseInfo bindInfo, long fence) {
        int result;
        try {
            lock.lock();
            result = VK10.vkQueueBindSparse(this.queue, bindInfo, fence);
        } finally {
            lock.unlock();
        }
        return result;
    }

    public int vkQueueBindSparse(VkBindSparseInfo.Buffer pBindInfo, long fence) {
        int result;
        try {
            lock.lock();
            result = VK10.vkQueueBindSparse(this.queue, pBindInfo, fence);
        } finally {
            lock.unlock();
        }
        return result;
    }

    // THIS IS BAD. VERY VERY BAD. DO NOT DO THIS. EVER... (but if for some reason you do need to here is a safe function)
    public int vkQueueWaitIdle() {
        int result;
        try {
            lock.lock();
            result = VK10.vkQueueWaitIdle(this.queue);
        } finally {
            lock.unlock();
        }
        return result;
    }

    public int vkQueuePresentKHR(VkPresentInfoKHR presentInfo) {
        int result;
        try {
            lock.lock();
            result = KHRSwapchain.vkQueuePresentKHR(this.queue, presentInfo);
        } finally {
            lock.unlock();
        }
        return result;
    }
}
