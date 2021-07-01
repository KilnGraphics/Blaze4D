package me.hydos.blaze4d.api.vertex;

import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.texture.UploadableImage;
import me.hydos.rosella.render.vertex.VertexConsumer;
import net.minecraft.client.render.VertexFormat;
import net.minecraft.util.math.Vec3f;
import org.joml.Matrix4f;
import org.joml.Vector3f;

import java.util.List;
import java.util.Objects;

/**
 * Contains information for rendering objects to the screen like a VertexConsumer and ModelViewMatrix
 */
public class ObjectInfo {
    public final ShaderProgram shader;
    public final VertexConsumer consumer;
    public final VertexFormat.DrawMode drawMode;
    public final VertexFormat format;
    public final UploadableImage image;
    public final Matrix4f projMatrix;
    public final Matrix4f viewMatrix;
    public final Vector3f chunkOffset;
    public final Vec3f shaderLightDirections0;
    public final Vec3f shaderLightDirections1;
    public List<Integer> indices;

    public ObjectInfo(VertexConsumer consumer, VertexFormat.DrawMode drawMode, VertexFormat format, ShaderProgram shader, UploadableImage image, Matrix4f projMatrix, Matrix4f viewMatrix, Vector3f chunkOffset, Vec3f shaderLightDirections0, Vec3f shaderLightDirections1, List<Integer> indices) {
        this.consumer = consumer;
        this.drawMode = drawMode;
        this.format = format;
        this.image = image;
        this.projMatrix = projMatrix;
        this.viewMatrix = viewMatrix;
        this.chunkOffset = chunkOffset;
        this.shaderLightDirections0 = shaderLightDirections0;
        this.shaderLightDirections1 = shaderLightDirections1;
        this.shader = shader;
        this.indices = indices;
    }


    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        ObjectInfo that = (ObjectInfo) o;
        return shader.equals(that.shader) && consumer.equals(that.consumer) && drawMode == that.drawMode && format.equals(that.format) && image.equals(that.image) && projMatrix.equals(that.projMatrix) && viewMatrix.equals(that.viewMatrix) && chunkOffset.equals(that.chunkOffset) && shaderLightDirections0.equals(that.shaderLightDirections0) && shaderLightDirections1.equals(that.shaderLightDirections1) && indices.equals(that.indices);
    }

    @Override
    public int hashCode() {
        return Objects.hash(shader, drawMode, format, image, projMatrix, viewMatrix, chunkOffset, shaderLightDirections0, shaderLightDirections1, consumer);
    }
}
