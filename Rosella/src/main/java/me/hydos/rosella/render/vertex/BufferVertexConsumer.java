package me.hydos.rosella.render.vertex;

import me.hydos.rosella.device.VulkanDevice;
import me.hydos.rosella.memory.Memory;

import java.nio.ByteBuffer;
import java.util.Collections;
import java.util.List;
import java.util.Objects;

// FIXME redo this whole class honestly
//public final class BufferVertexConsumer {
//    private final VertexFormat format;
//
//    private ByteBuffer buffer;
//    private int vertexCount;
//    private int debugSize;
//
//    public BufferVertexConsumer(VertexFormat format) {
//        this.format = format;
//        this.buffer = null;
//    }
//
//    public List<ManagedBuffer> getBuffers() {
//        return Collections.singletonList(new ManagedBuffer(buffer, 0, 0, vertexCount + format.getSize(), false));
//    }
//
//    public VertexFormat getFormat() {
//        return format;
//    }
//
//    public BufferVertexConsumer pos(final float x, final float y, final float z) {
//        this.buffer.putFloat(x);
//        this.buffer.putFloat(y);
//        this.buffer.putFloat(z);
//        this.debugSize += 12;
//        return this;
//    }
//
//    public BufferVertexConsumer color(final byte red, final byte green, final byte blue) {
//        this.buffer.put(red);
//        this.buffer.put(green);
//        this.buffer.put(blue);
//        this.debugSize += 3;
//        return this;
//    }
//
//    public BufferVertexConsumer color(final byte red, final byte green, final byte blue, final byte alpha) {
//        this.buffer.put(red);
//        this.buffer.put(green);
//        this.buffer.put(blue);
//        this.buffer.put(alpha);
//        this.debugSize += 4;
//        return this;
//    }
//
//    public BufferVertexConsumer normal(final float x, final float y, final float z) {
//        this.buffer.putFloat(x);
//        this.buffer.putFloat(y);
//        this.buffer.putFloat(z);
//        this.debugSize += 12;
//        return this;
//    }
//
//    public BufferVertexConsumer uv(final float u, final float v) {
//        this.buffer.putFloat(u);
//        this.buffer.putFloat(v);
//        this.debugSize += 8;
//        return this;
//    }
//
//    public BufferVertexConsumer uv(final short u, final short v) {
//        this.buffer.putShort(u);
//        this.buffer.putShort(v);
//        this.debugSize += 4;
//        return this;
//    }
//
//    public BufferVertexConsumer putByte(final int index, final byte value) {
//        this.buffer.put(index, value);
//        this.debugSize++;
//        return this;
//    }
//
//    public BufferVertexConsumer putShort(final int index, final short value) {
//        this.buffer.putShort(index, value);
//        this.debugSize += 2;
//        return this;
//    }
//
//    public BufferVertexConsumer putFloat(final int index, final float value) {
//        this.buffer.putFloat(index, value);
//        this.debugSize += 4;
//        return this;
//    }
//
//    public BufferVertexConsumer nextVertex() {
//        if (this.debugSize != this.format.getSize()) {
//            throw new RuntimeException("Incorrect vertex size passed. Received " + this.debugSize + " but wanted " + this.format.getSize());
//        } else {
//            this.debugSize = 0;
//            this.vertexCount++;
//            return this;
//        }
//    }
//
//    public void clear() {
//        this.buffer.clear(); // FIXME make a new buffer
//        this.vertexCount = 0;
//    }
//
//    public void free(VulkanDevice device, Memory memory) {
//        // FIXME
//    }
//
//    public int getVertexSize() {
//        return this.format.getSize();
//    }
//
//    public int getVertexCount() {
//        return this.vertexCount;
//    }
//
//    @Override
//    public boolean equals(Object o) {
//        if (this == o) return true;
//        if (o == null || getClass() != o.getClass()) return false;
//        BufferVertexConsumer that = (BufferVertexConsumer) o;
//        return vertexCount == that.vertexCount && debugSize == that.debugSize && format.equals(that.format) && buffer.equals(that.buffer);
//    }
//
//    @Override
//    public int hashCode() {
//        return Objects.hash(format, buffer, vertexCount, debugSize);
//    }
//}
