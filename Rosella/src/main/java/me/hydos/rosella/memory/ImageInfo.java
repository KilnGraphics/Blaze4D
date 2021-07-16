package me.hydos.rosella.memory;

import me.hydos.rosella.device.VulkanDevice;

public record ImageInfo(long buffer, long allocation) implements MemoryCloseable {
    @Override
    public void free(VulkanDevice device, Memory memory) {
        memory.freeImage(this);
    }
}