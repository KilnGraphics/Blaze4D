package me.hydos.rosella.memory;

import me.hydos.rosella.device.VulkanDevice;

public record BufferInfo(long buffer, long allocation) implements MemoryCloseable {
    @Override
    public void free(VulkanDevice device, Memory memory) {
        memory.freeBuffer(this);
    }
}
