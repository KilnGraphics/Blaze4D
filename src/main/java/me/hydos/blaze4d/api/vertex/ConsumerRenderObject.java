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
import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.texture.Texture;
import me.hydos.rosella.render.vertex.BufferVertexConsumer;
import me.hydos.rosella.render.vertex.VertexFormats;
import me.hydos.rosella.scene.object.Renderable;
import net.minecraft.client.render.VertexFormat;
import org.jetbrains.annotations.NotNull;

public class ConsumerRenderObject implements Renderable {

    private final VertexFormat format;
    private final Texture[] textures;
    private final ShaderProgram shader;

    // Render Implementation Fields
    public final RenderInfo renderInfo = new RenderInfo(new BufferVertexConsumer(VertexFormats.POSITION_COLOR3_UV));
    public InstanceInfo instanceInfo;

    public ConsumerRenderObject(ObjectInfo info, Rosella rosella) {
        this.renderInfo.consumer = info.consumer();
        VertexFormat.DrawMode drawMode = info.drawMode();
        this.format = info.format();
        this.shader = info.shader();
        this.textures = info.textures();
        Material material = getMaterial(drawMode);
        instanceInfo = new InstanceInfo(((MinecraftShaderProgram) info.shader().getRaw()).createMinecraftUbo(rosella.common.memory, material), material);
        ((MinecraftUbo) instanceInfo.ubo()).setUniforms(info.projMatrix(), info.viewMatrix(), info.chunkOffset(), info.shaderLightDirections0(), info.shaderLightDirections1());
        this.renderInfo.indices = info.indices();
    }

    private Material getMaterial(VertexFormat.DrawMode drawMode) {
        Material returnValue = null;
        switch (drawMode) {
            case TRIANGLES, QUADS -> {
                if (format != net.minecraft.client.render.VertexFormats.BLIT_SCREEN) {
                    returnValue = Materials.TRIANGLES.build(shader, textures, renderInfo.consumer.getFormat());
                }
            }

            case TRIANGLE_STRIP, DEBUG_LINE_STRIP -> {
                if (format == net.minecraft.client.render.VertexFormats.POSITION) {
                    returnValue = Materials.TRIANGLE_STRIP.build(shader, textures, renderInfo.consumer.getFormat());
                }
            }

            case TRIANGLE_FAN -> returnValue = Materials.TRIANGLE_FAN.build(shader, textures, renderInfo.consumer.getFormat());

            case LINES, DEBUG_LINES -> returnValue = Materials.LINES.build(shader, textures, renderInfo.consumer.getFormat());

            default -> throw new RuntimeException("Unsupported Draw Mode:  " + drawMode);
        }
        return returnValue;
    }

    //======================
    // Render Implementation
    //======================

    @Override
    public void onAddedToScene(Rosella rosella) {
        instanceInfo.rebuild(rosella);
        instanceInfo.ubo().create(rosella.renderer.swapchain);
    }

    @Override
    public void free(@NotNull Memory memory, @NotNull VulkanDevice  device) {
        instanceInfo.free(device, memory);
        renderInfo.free(device, memory);
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
    public RenderInfo getRenderInfo() {
        return renderInfo;
    }
}
