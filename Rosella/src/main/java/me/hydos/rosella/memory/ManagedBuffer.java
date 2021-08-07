package me.hydos.rosella.memory;

import me.hydos.rosella.device.LegacyVulkanDevice;

import java.nio.Buffer;

public record ManagedBuffer<T extends Buffer>(T buffer, boolean freeable) implements MemoryCloseable {

    @Override
    public void free(LegacyVulkanDevice device, Memory memory) {
        if (freeable) {
            memory.freeDirectBufferAsync(buffer);
        }
    }
}
