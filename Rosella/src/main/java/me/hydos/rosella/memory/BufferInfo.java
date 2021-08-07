package me.hydos.rosella.memory;

import me.hydos.rosella.device.LegacyVulkanDevice;

public record BufferInfo(long buffer, long allocation) implements MemoryCloseable {
    @Override
    public void free(LegacyVulkanDevice device, Memory memory) {
        memory.freeBuffer(this);
    }
}
