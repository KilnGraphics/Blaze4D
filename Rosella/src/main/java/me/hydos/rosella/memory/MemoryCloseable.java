package me.hydos.rosella.memory;

import me.hydos.rosella.device.LegacyVulkanDevice;

/**
 * Used to safely close memory when an object wants to be de-allocated.
 */
public interface MemoryCloseable {
    void free(LegacyVulkanDevice device, Memory memory);
}
