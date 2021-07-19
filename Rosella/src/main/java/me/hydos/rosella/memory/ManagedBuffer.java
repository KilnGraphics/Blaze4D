package me.hydos.rosella.memory;

import me.hydos.rosella.device.VulkanDevice;
import org.lwjgl.system.MemoryUtil;

import java.nio.Buffer;

public record ManagedBuffer<T extends Buffer>(T buffer, boolean freeable) {

    // this doesn't implement MemoryClosable because it's not vulkan specific and we don't need Memory or VulkanDevice
    public void tryFree() {
        if (freeable) {
            MemoryUtil.memFree(buffer);
        }
    }
}
