package me.hydos.rosella.render.vertex;

import it.unimi.dsi.fastutil.objects.ObjectArrayList;
import org.lwjgl.system.MemoryUtil;

import java.nio.ByteBuffer;
import java.util.List;

public class StoredBufferProvider implements BufferProvider {

    private final VertexFormat format;
    private final List<ManagedBuffer> buffers;
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
    public List<ManagedBuffer> getBuffers() {
        return buffers;
    }

    @Override
    public void clear() {
        for (ManagedBuffer buffer : buffers) {
            if (buffer.shouldFree()) {
                MemoryUtil.memFree(buffer.buffer());
            }
        }
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

    public void addBuffer(ByteBuffer byteBuffer, int posOffset, int vertexCount, boolean shouldFree) {
        buffers.add(new ManagedBuffer(byteBuffer, posOffset, totalVertexCount * getVertexSize(), vertexCount * getVertexSize(), shouldFree));
        totalVertexCount += vertexCount;
    }
}
