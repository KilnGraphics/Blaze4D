package me.hydos.rosella.render.vertex;

import me.hydos.rosella.memory.MemoryCloseable;

import java.nio.ByteBuffer;
import java.util.List;

public interface BufferProvider extends MemoryCloseable {
    VertexFormat getFormat();

    List<ManagedBuffer> getBuffers();

    void clear();

    int getVertexSize();

    int getVertexCount();

    record ManagedBuffer(ByteBuffer buffer, int srcPos, int dstPos, int length, boolean shouldFree) {}
}
