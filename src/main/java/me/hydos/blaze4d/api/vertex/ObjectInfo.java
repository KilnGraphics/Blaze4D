package me.hydos.blaze4d.api.vertex;

import com.mojang.blaze3d.vertex.VertexFormat;
import it.unimi.dsi.fastutil.ints.IntList;
import me.hydos.rosella.render.material.state.StateInfo;
import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.texture.Texture;
import me.hydos.rosella.render.vertex.BufferProvider;
import org.joml.Matrix4f;
import org.joml.Vector3f;

/**
 * Contains information for rendering objects to the screen like a VertexConsumer and ModelViewMatrix
 */
public record ObjectInfo(BufferProvider bufferProvider,
                         VertexFormat.Mode drawMode,
                         VertexFormat format,
                         ShaderProgram shader,
                         Texture[] textures,
                         StateInfo stateInfo, Matrix4f projMatrix,
                         Matrix4f viewMatrix, Vector3f chunkOffset,
                         com.mojang.math.Vector3f shaderLightDirections0,
                         com.mojang.math.Vector3f shaderLightDirections1,
                         IntList indices) {
}
