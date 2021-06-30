package me.hydos.rosella.render.info;

import me.hydos.rosella.Rosella;
import me.hydos.rosella.memory.MemoryCloseable;
import me.hydos.rosella.render.device.Device;
import me.hydos.rosella.render.util.memory.BufferInfo;
import me.hydos.rosella.render.util.memory.Memory;
import me.hydos.rosella.render.vertex.VertexConsumer;
import org.jetbrains.annotations.NotNull;

import java.util.List;

/**
 * Info like the consumer used. this is usually tied to a RenderableObject with instance info. if an object can be instanced, it will be bound once and transformations will be applied separately
 */
public class RenderInfo implements MemoryCloseable {

    public VertexConsumer consumer;
    public List<Integer> indices;

    private BufferInfo vertexBuffer;
    private BufferInfo indexBuffer;

    public RenderInfo(@NotNull VertexConsumer consumer) {
        this.consumer = consumer;
    }

    /**
     * If the RenderInfo is indeed unique to the current scene, an Vertex and Index buffer will be created
     */
    public void createBuffers(Memory memory, Rosella rosella) {
        vertexBuffer = memory.createVertexBuffer(rosella, consumer);
        indexBuffer = memory.createIndexBuffer(rosella, indices);
    }

    @Override
    public boolean equals(Object obj) {
        if (obj instanceof RenderInfo) {
            return consumer.equals(((RenderInfo) obj).consumer);
        }
        return false;
    }

    /**
     * A safe way to get access to buffers. this method will throw a {@link RuntimeException} if the vertex buffer is null
     *
     * @return a {@link BufferInfo} with Vertex data
     */
    public BufferInfo getVertexBuffer() {
        if (vertexBuffer == null) {
            throw new RuntimeException("Tried to access buffers when not set. (This is probably an internal error)");
        }
        return vertexBuffer;
    }

    /**
     * A safe way to get access to buffers. this method will throw a {@link RuntimeException} if the index buffer is null
     *
     * @return a {@link BufferInfo} with Index data
     */
    public BufferInfo getIndexBuffer() {
        if (indexBuffer == null) {
            throw new RuntimeException("Tried to access buffers when not set. (This is probably an internal error)");
        }
        return indexBuffer;
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
    public void free(Device device, Memory memory) {
        memory.freeBuffer(vertexBuffer);
        memory.freeBuffer(indexBuffer);
    }
}
