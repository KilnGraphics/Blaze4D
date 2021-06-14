package me.hydos.blaze4d.api.vertex;

import me.hydos.blaze4d.api.util.Vector2s;
import net.fabricmc.api.EnvType;
import net.fabricmc.api.Environment;
import net.minecraft.client.render.VertexFormatElement;
import org.joml.Vector2f;
import org.joml.Vector3f;
import org.joml.Vector4f;

import java.nio.ByteBuffer;

/**
 * Supplies A Vertex :)
 * Mojang All You do is cause pain by using those weird buffer's
 */
@Environment(EnvType.CLIENT)
public interface VertexSupplier {

    Vector3f vertex(ByteBuffer vertexBuffer, VertexFormatElement info);

    Vector4f color(ByteBuffer vertexBuffer, VertexFormatElement info);

    Vector2f texture(ByteBuffer vertexBuffer, VertexFormatElement info);

    Vector2s overlay(ByteBuffer vertexBuffer);

    int light(ByteBuffer vertexBuffer);

    Vector3f normal(ByteBuffer vertexBuffer, VertexFormatElement info);

    void padding(ByteBuffer vertBuf, VertexFormatElement info);
}
