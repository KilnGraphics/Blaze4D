package me.hydos.rosella.memory;

import me.hydos.rosella.device.VulkanDevice;
import org.lwjgl.util.vma.Vma;

public record BufferInfo(long buffer, long allocation) implements MemoryCloseable {
    @Override
    public void free(VulkanDevice device, Memory memory) {
        memory.freeBuffer(this);
    }

    @Override
    public void free(long allocator) {
        Vma.vmaDestroyBuffer(allocator, buffer, allocation);
    }
}
