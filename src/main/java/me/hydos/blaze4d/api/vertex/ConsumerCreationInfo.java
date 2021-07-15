package me.hydos.blaze4d.api.vertex;

import com.mojang.blaze3d.vertex.VertexFormat;
import me.hydos.rosella.render.material.state.StateInfo;
import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.texture.Texture;
import org.joml.Matrix4f;
import org.joml.Vector3f;

import java.util.Arrays;
import java.util.Objects;

public record ConsumerCreationInfo(VertexFormat.Mode drawMode, VertexFormat format, ShaderProgram shader, Texture[] textures,
                                   StateInfo stateInfo, Matrix4f projMatrix, Matrix4f viewMatrix, Vector3f chunkOffset,
                                   com.mojang.math.Vector3f shaderLightDirections0, com.mojang.math.Vector3f shaderLightDirections1) {
    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        ConsumerCreationInfo that = (ConsumerCreationInfo) o;
        return drawMode == that.drawMode && format.equals(that.format) && Arrays.equals(textures, that.textures) && shader.equals(that.shader) && stateInfo.equals(that.stateInfo) && projMatrix.equals(that.projMatrix) && viewMatrix.equals(that.viewMatrix) && chunkOffset.equals(that.chunkOffset) && shaderLightDirections0.equals(that.shaderLightDirections0) && shaderLightDirections1.equals(that.shaderLightDirections1);
    }

    @Override
    public int hashCode() {
        int result = Objects.hash(drawMode, format, shader, stateInfo, projMatrix, viewMatrix, chunkOffset, shaderLightDirections0, shaderLightDirections1);
        result = 31 * result + Arrays.hashCode(textures);
        return result;
    }
}
