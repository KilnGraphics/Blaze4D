package me.hydos.rosella.render.info;

import it.unimi.dsi.fastutil.ints.IntList;
import me.hydos.rosella.device.VulkanDevice;
import me.hydos.rosella.memory.Memory;
import me.hydos.rosella.memory.MemoryCloseable;
import me.hydos.rosella.render.vertex.BufferProvider;
import org.jetbrains.annotations.NotNull;

import java.util.List;

/**
 * Info like the consumer used. this is usually tied to a RenderableObject with instance info. if an object can be instanced, it will be bound once and transformations will be applied separately
 */
public class RenderInfo implements MemoryCloseable {

    public BufferProvider bufferProvider;
    public IntList indices;

    public RenderInfo(@NotNull BufferProvider bufferProvider) {
        this.bufferProvider = bufferProvider;
    }

    @Override
    public boolean equals(Object obj) {
        if (obj instanceof RenderInfo) {
            return bufferProvider.equals(((RenderInfo) obj).bufferProvider);
        }
        return false;
    }

    /**
     * Gets the size of the index array
     *
     * @return the size of the index array
     */
    public int getIndicesSize() {
        if (indices.size() <= 0) {
            throw new RuntimeException("Tried to render with 0 indices");
        }
        return indices.size();
    }

    @Override
    public void free(VulkanDevice device, Memory memory) {
    }
}
