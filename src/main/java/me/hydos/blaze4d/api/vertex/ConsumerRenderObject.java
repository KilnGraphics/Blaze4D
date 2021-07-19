package me.hydos.blaze4d.api.vertex;

import me.hydos.blaze4d.api.Materials;
import me.hydos.blaze4d.api.shader.MinecraftShaderProgram;
import me.hydos.blaze4d.api.shader.MinecraftUbo;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.device.VulkanDevice;
import me.hydos.rosella.memory.Memory;
import me.hydos.rosella.render.info.InstanceInfo;
import me.hydos.rosella.render.info.RenderInfo;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.material.state.StateInfo;
import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.texture.Texture;
import me.hydos.rosella.render.vertex.VertexFormat;
import me.hydos.rosella.scene.object.Renderable;
import org.joml.Matrix4f;
import org.joml.Vector3f;

import java.util.concurrent.ExecutionException;
import java.util.concurrent.Future;

public class ConsumerRenderObject implements Renderable {

    // Render Implementation Fields
    private final Future<RenderInfo> renderInfo;
    private final VertexFormat format;
    private final Texture[] textures;
    private final ShaderProgram shader;
    private final StateInfo stateInfo;
    public InstanceInfo instanceInfo;

    public ConsumerRenderObject(
            Future<RenderInfo> renderInfo,
            VertexFormat format,
            ShaderProgram shader,
            Texture[] textures,
            StateInfo stateInfo,
            Matrix4f projMatrix,
            Matrix4f viewMatrix,
            Vector3f chunkOffset,
            com.mojang.math.Vector3f shaderLightDirections0,
            com.mojang.math.Vector3f shaderLightDirections1,
            com.mojang.blaze3d.vertex.VertexFormat mcFormat,
            com.mojang.blaze3d.vertex.VertexFormat.Mode mcDrawMode,
            Rosella rosella) {

        this.renderInfo = renderInfo;
        this.format = format;
        this.shader = shader;
        this.textures = textures;
        this.stateInfo = stateInfo;
        Material material = getMaterial(mcFormat, mcDrawMode);
        instanceInfo = new InstanceInfo(((MinecraftShaderProgram) shader.getRaw()).createMinecraftUbo(rosella.common.memory, material), material);
        ((MinecraftUbo) instanceInfo.ubo()).setUniforms(projMatrix, viewMatrix, chunkOffset, shaderLightDirections0, shaderLightDirections1);
    }

    private Material getMaterial(com.mojang.blaze3d.vertex.VertexFormat mcFormat, com.mojang.blaze3d.vertex.VertexFormat.Mode mcDrawMode) {
        Material returnValue = null;
        switch (mcDrawMode) {
            case TRIANGLES, QUADS, LINES -> {
                if (mcFormat != com.mojang.blaze3d.vertex.DefaultVertexFormat.BLIT_SCREEN) { // TODO: why?
                    returnValue = Materials.TRIANGLES.build(shader, textures, format, stateInfo);
                }
            }

            case TRIANGLE_STRIP, LINE_STRIP -> {
                if (mcFormat == com.mojang.blaze3d.vertex.DefaultVertexFormat.POSITION) { // TODO: why?
                    returnValue = Materials.TRIANGLE_STRIP.build(shader, textures, format, stateInfo);
                }
            }

            case TRIANGLE_FAN -> returnValue = Materials.TRIANGLE_FAN.build(shader, textures, format, stateInfo);

            case DEBUG_LINES -> returnValue = Materials.LINES.build(shader, textures, format, stateInfo);

            case DEBUG_LINE_STRIP -> returnValue = Materials.LINE_STRIP.build(shader, textures, format, stateInfo);

            default -> throw new RuntimeException("Unsupported Draw Mode:  " + mcDrawMode);
        }
        return returnValue;
    }

    //======================
    // Render Implementation
    //======================

    @Override
    public void onAddedToScene(Rosella rosella) {
        instanceInfo.hardRebuild(rosella);
    }

    @Override
    public void free(VulkanDevice device, Memory memory) {
        instanceInfo.free(device, memory);
        try {
            renderInfo.get().free(device, memory);
        } catch (InterruptedException | ExecutionException e) {
            Rosella.LOGGER.error("Error freeing render info", e);
        }
    }

    @Override
    public void rebuild(Rosella rosella) {
        instanceInfo.rebuild(rosella);
    }

    @Override
    public InstanceInfo getInstanceInfo() {
        return instanceInfo;
    }

    @Override
    public Future<RenderInfo> getRenderInfo() {
        return renderInfo;
    }
}
