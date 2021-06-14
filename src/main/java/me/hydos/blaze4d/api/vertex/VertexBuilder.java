package me.hydos.blaze4d.api.vertex;

import me.hydos.blaze4d.api.util.Vector2s;
import net.minecraft.client.render.VertexFormatElement;
import org.joml.Vector2f;
import org.joml.Vector3f;
import org.joml.Vector4f;

import java.nio.ByteBuffer;

/**
 * Builds Vertices. Nothing much to say about it
 */
public class VertexBuilder implements VertexSupplier {

    public int index;
    private int elementOffset;

    @Override
    public Vector3f vertex(ByteBuffer vertexBuffer, VertexFormatElement info) {
        float x = vertexBuffer.getFloat(index);
        float y = vertexBuffer.getFloat(index + 4);
        float z = vertexBuffer.getFloat(index + 8);
        index += 12;
        return new Vector3f(x, y, z);
    }

    @Override
    public Vector4f color(ByteBuffer vertexBuffer, VertexFormatElement info) {
        float r = vertexBuffer.get(index) / 255f;
        float g = vertexBuffer.get(index + 1) / 255f;
        float b = vertexBuffer.get(index + 2) / 255f;
        float a = vertexBuffer.get(index + 3) / 255f;
        index += 3;
        return new Vector4f(r, g, b, a);
    }

    @Override
    public Vector2f texture(ByteBuffer vertexBuffer, VertexFormatElement info) {
        float u = 0;
        float v = 0;

        switch (info.getDataType()) {
            case FLOAT -> {
                u = vertexBuffer.getFloat(index);
                v = vertexBuffer.getFloat(index + 4);
                index += 8;
            }

            case SHORT -> {
                u = vertexBuffer.getShort(index);
                v = vertexBuffer.getShort(index + 4);
                index += 4;
            }
        }

        return new Vector2f(u, v);
    }

    @Override
    public Vector2s overlay(ByteBuffer vertexBuffer) {
        short u = vertexBuffer.getShort(index);
        short v = vertexBuffer.getShort(index + 2);
        index += 4;
        return new Vector2s(u, v);
    }

    @Override
    public int light(ByteBuffer vertexBuffer) {
        short u = vertexBuffer.getShort(index);
        short v = vertexBuffer.getShort(index + 2);
        index += 4;
        return u | v << 16;
    }

    @Override
    public Vector3f normal(ByteBuffer vertexBuffer, VertexFormatElement info) {
        float normalX = vertexBuffer.get(index) * 255f;
        float normalY = vertexBuffer.get(index + 1) * 255f;
        float normalZ = vertexBuffer.get(index + 2) * 255f;
        index += 3;
        return new Vector3f(normalX, normalY, normalZ);
    }

    @Override
    public void padding(ByteBuffer vertBuf, VertexFormatElement info) {
        vertBuf.get(index);
        index += 1;
    }

    public ByteBuffer next(ByteBuffer vertBuffer) {
        elementOffset = index;
        return vertBuffer.position(this.elementOffset);
    }

    public int getElementOffset() {
        return elementOffset;
    }
}
