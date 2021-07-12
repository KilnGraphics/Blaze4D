package me.hydos.blaze4d.api.vertex;

import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.texture.Texture;
import net.minecraft.client.render.VertexFormat;
import net.minecraft.util.math.Vec3f;
import org.joml.Matrix4f;
import org.joml.Vector3f;

import java.util.Arrays;
import java.util.Objects;

public record ConsumerCreationInfo(VertexFormat.DrawMode drawMode, VertexFormat format, Texture[] textures,
                                   ShaderProgram shader, Matrix4f projMatrix, Matrix4f viewMatrix, Vector3f chunkOffset,
                                   Vec3f shaderLightDirections0, Vec3f shaderLightDirections1) {
    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        ConsumerCreationInfo that = (ConsumerCreationInfo) o;
        return drawMode == that.drawMode && format.equals(that.format) && Arrays.equals(textures, that.textures) && shader.equals(that.shader) && projMatrix.equals(that.projMatrix) && viewMatrix.equals(that.viewMatrix) && chunkOffset.equals(that.chunkOffset) && shaderLightDirections0.equals(that.shaderLightDirections0) && shaderLightDirections1.equals(that.shaderLightDirections1);
    }

    @Override
    public int hashCode() {
        int result = Objects.hash(drawMode, format, shader, projMatrix, viewMatrix, chunkOffset, shaderLightDirections0, shaderLightDirections1);
        result = 31 * result + Arrays.hashCode(textures);
        return result;
    }
}
