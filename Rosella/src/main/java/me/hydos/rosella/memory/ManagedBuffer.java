package me.hydos.rosella.memory;

import me.hydos.rosella.device.VulkanDevice;

import java.nio.Buffer;

public record ManagedBuffer<T extends Buffer>(T buffer, boolean freeable) implements MemoryCloseable {

    @Override
    public void free(VulkanDevice device, Memory memory) {
        if (freeable) {
            memory.freeDirectBufferAsync(buffer);
        }
    }
}
