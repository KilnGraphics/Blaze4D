package me.hydos.blaze4d.api.vertex;

import java.util.Arrays;
import java.util.Objects;

import com.google.common.collect.ImmutableList;
import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.texture.Texture;
import org.joml.Matrix4f;
import org.joml.Vector3f;

import net.minecraft.client.render.VertexFormat;
import net.minecraft.client.render.VertexFormatElement;
import net.minecraft.util.math.Vec3f;

public record ConsumerCreationInfo(VertexFormat.DrawMode drawMode, VertexFormat format, ImmutableList<VertexFormatElement> elements, Texture[] textures, ShaderProgram shader, Matrix4f projMatrix, Matrix4f viewMatrix, Vector3f chunkOffset, Vec3f shaderLightDirections0, Vec3f shaderLightDirections1) {
    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        ConsumerCreationInfo that = (ConsumerCreationInfo) o;
        return drawMode == that.drawMode && Objects.equals(format, that.format) && Objects.equals(elements, that.elements) && Arrays.equals(textures, that.textures) && Objects.equals(shader, that.shader) && Objects.equals(projMatrix, that.projMatrix) && Objects.equals(viewMatrix, that.viewMatrix) && Objects.equals(chunkOffset, that.chunkOffset) && Objects.equals(shaderLightDirections0, that.shaderLightDirections0) && Objects.equals(shaderLightDirections1, that.shaderLightDirections1);
    }
}
