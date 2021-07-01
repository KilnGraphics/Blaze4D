package me.hydos.blaze4d.api.vertex;

import me.hydos.blaze4d.api.Materials;
import me.hydos.blaze4d.api.shader.MinecraftUbo;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.render.device.Device;
import me.hydos.rosella.render.info.InstanceInfo;
import me.hydos.rosella.render.info.RenderInfo;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.object.Renderable;
import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.texture.UploadableImage;
import me.hydos.rosella.render.util.memory.Memory;
import me.hydos.rosella.render.vertex.BufferVertexConsumer;
import me.hydos.rosella.render.vertex.VertexConsumer;
import me.hydos.rosella.render.vertex.VertexFormats;
import net.minecraft.client.render.VertexFormat;
import net.minecraft.util.math.Vec3f;
import org.jetbrains.annotations.NotNull;
import org.joml.Matrix4f;
import org.joml.Vector3f;

public class ConsumerRenderObject implements Renderable {

    private final net.minecraft.client.render.VertexFormat.DrawMode drawMode;
    private final VertexFormat format;
    private final int textureId;
    private final ShaderProgram shader;

    // Render Implementation Fields
    public final RenderInfo renderInfo = new RenderInfo(new BufferVertexConsumer(VertexFormats.Companion.getPOSITION_COLOR_UV()));
    public InstanceInfo instanceInfo;

    public ConsumerRenderObject(ObjectInfo info, Rosella rosella) {
        this.renderInfo.consumer = info.consumer;
        this.drawMode = info.drawMode;
        this.format = info.format;
        this.shader = info.shader;
        this.textureId = info.textureId;
        Material material = getMaterial(drawMode);
        instanceInfo = new InstanceInfo(new MinecraftUbo(rosella.getDevice(), rosella.getMemory(), material), material);
        ((MinecraftUbo) instanceInfo.ubo).setUniforms(info.projMatrix, info.viewMatrix, info.chunkOffset, info.shaderLightDirections0, info.shaderLightDirections1);
        this.renderInfo.indices = info.indices;
    }

    private Material getMaterial(VertexFormat.DrawMode drawMode) {
        Material returnValue = null;
        switch (drawMode) {
            case TRIANGLES, QUADS -> {
                if (format != net.minecraft.client.render.VertexFormats.BLIT_SCREEN) {
                    returnValue = Materials.TRIANGLES.build(shader, textureId, renderInfo.consumer.getFormat());
                }
            }

            case TRIANGLE_STRIP -> {
                if (format == net.minecraft.client.render.VertexFormats.POSITION) {
                    returnValue = Materials.TRIANGLE_STRIP.build(shader, textureId, renderInfo.consumer.getFormat());
                }
            }

            case TRIANGLE_FAN -> returnValue = Materials.TRIANGLE_FAN.build(shader, textureId, renderInfo.consumer.getFormat());

            case LINES -> returnValue = Materials.LINES.build(shader, textureId, renderInfo.consumer.getFormat());

            default -> throw new RuntimeException("Unsupported Draw Mode:  " + drawMode);
        }
        return returnValue;
    }

    //======================
    // Render Implementation
    //======================


    public void onAddedToScene(@NotNull Rosella rosella) {
        instanceInfo.rebuild(rosella);
        instanceInfo.ubo.create(rosella.getRenderer().swapchain);
    }

    @Override
    public void free(@NotNull Memory memory, @NotNull Device device) {
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
