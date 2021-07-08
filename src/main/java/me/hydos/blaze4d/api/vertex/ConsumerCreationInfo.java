package me.hydos.blaze4d.api.vertex;

import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.texture.Texture;
import net.minecraft.client.render.VertexFormat;
import net.minecraft.util.math.Vec3f;
import org.joml.Matrix4f;
import org.joml.Vector3f;

public record ConsumerCreationInfo(VertexFormat.DrawMode drawMode, VertexFormat format, Texture[] textures, ShaderProgram shader, Matrix4f projMatrix, Matrix4f viewMatrix, Vector3f chunkOffset, Vec3f shaderLightDirections0, Vec3f shaderLightDirections1) {
}
