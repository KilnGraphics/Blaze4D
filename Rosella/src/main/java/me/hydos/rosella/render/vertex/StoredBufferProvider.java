package me.hydos.rosella.render.vertex;

import it.unimi.dsi.fastutil.objects.ObjectArrayList;

import java.nio.ByteBuffer;
import java.util.List;

public class StoredBufferProvider implements BufferProvider {

    private final VertexFormat format;
    private final List<PositionedBuffer> buffers;
    private int totalVertexCount;

    public StoredBufferProvider(VertexFormat format) {
        this.format = format;
        this.buffers = new ObjectArrayList<>();
    }

    @Override
    public VertexFormat getFormat() {
        return format;
    }

    @Override
    public List<PositionedBuffer> getBuffers() {
        return buffers;
    }

    @Override
    public void clear() {
        // TODO: should we be doing this?
//        for (PositionedBuffer buffer : buffers) {
//            buffer.buffer().clear();
//        }
        buffers.clear();
    }

    @Override
    public int getVertexSize() {
        return format.getSize();
    }

    @Override
    public int getVertexCount() {
        return totalVertexCount;
    }

    public void addBuffer(ByteBuffer byteBuffer, int posOffset, int vertexCount) {
        buffers.add(new PositionedBuffer(byteBuffer, posOffset, totalVertexCount * getVertexSize(), vertexCount * getVertexSize()));
        totalVertexCount += vertexCount;
    }
}
