package me.hydos.rosella.memory.buffer;

import me.hydos.rosella.Rosella;
import me.hydos.rosella.memory.BufferInfo;
import me.hydos.rosella.memory.Memory;
import me.hydos.rosella.render.info.RenderInfo;

import java.util.List;

import static org.lwjgl.util.vma.Vma.VMA_MEMORY_USAGE_CPU_ONLY;
import static org.lwjgl.vulkan.VK10.VK_BUFFER_USAGE_TRANSFER_SRC_BIT;

/**
 * To do what all the cool kids do, we make 2 big nut buffers TM
 */
public class GlobalBufferManager {

    private final Memory memory;

    public GlobalBufferManager(Rosella rosella) {
        this.memory = rosella.common.memory;
    }

    /**
     * Creates a vertex buffer based on the RenderInfo parsed in
     * @param renderList the list of RenderInfo objects
     * @return a vertex buffer
     */
//    public BufferInfo createVertexBuffer(List<RenderInfo> renderList) {
//
//    }
}
