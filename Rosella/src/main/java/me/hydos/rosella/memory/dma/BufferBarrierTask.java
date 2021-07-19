package me.hydos.rosella.memory.dma;

import org.lwjgl.vulkan.VkBufferMemoryBarrier;

public interface BufferBarrierTask {

    void fillBarrier(VkBufferMemoryBarrier barrier);
}
