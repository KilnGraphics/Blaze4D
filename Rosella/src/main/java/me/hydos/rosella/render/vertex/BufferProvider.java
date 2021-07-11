package me.hydos.rosella.render.vertex;

import java.nio.ByteBuffer;
import java.util.List;

public interface BufferProvider {
    VertexFormat getFormat();

    List<PositionedBuffer> getBuffers();

    void clear();

    int getVertexSize();

    int getVertexCount();

    record PositionedBuffer(ByteBuffer buffer, int startPos, int length) {}
}
