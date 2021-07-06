package me.hydos.rosella.memory;

import me.hydos.rosella.device.VulkanDevice;
import me.hydos.rosella.render.util.memory.Memory;

/**
 * Used to safely close memory when an object wants to be de-allocated.
 */
public interface MemoryCloseable {

    void free(VulkanDevice device, Memory memory);
}
