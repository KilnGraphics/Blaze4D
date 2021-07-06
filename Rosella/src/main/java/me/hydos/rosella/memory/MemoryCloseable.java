package me.hydos.rosella.memory;

import me.hydos.rosella.device.VulkanDevice;

/**
 * Used to safely close memory when an object wants to be de-allocated.
 */
public interface MemoryCloseable {
    void free(VulkanDevice device, Memory memory);

    void free(long allocator);
}
