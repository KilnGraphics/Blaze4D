package me.hydos.blaze4d.api.vertex;

import me.hydos.blaze4d.api.Materials;
import me.hydos.blaze4d.api.shader.MinecraftUbo;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.render.descriptorsets.DescriptorSet;
import me.hydos.rosella.render.device.Device;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.model.Renderable;
import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.shader.ubo.Ubo;
import me.hydos.rosella.render.texture.UploadableImage;
import me.hydos.rosella.render.util.memory.BufferInfo;
import me.hydos.rosella.render.util.memory.Memory;
import me.hydos.rosella.render.vertex.VertexConsumer;
import net.minecraft.client.render.VertexFormat;
import org.jetbrains.annotations.NotNull;
import org.joml.Matrix4f;

import java.util.ArrayList;
import java.util.List;

public class ConsumerRenderObject implements Renderable {

    private final VertexConsumer consumer;
    private final net.minecraft.client.render.VertexFormat.DrawMode drawMode;
    private final VertexFormat format;
    private final UploadableImage image;
    private final ShaderProgram shader;

    // Renderable Fields
    private final Matrix4f transformationMatrix = new Matrix4f();
    private final MinecraftUbo ubo;
    public Material material;
    public List<Integer> indices = new ArrayList<>();
    public BufferInfo vertexBuffer = null;
    public BufferInfo indexBuffer = null;
    public DescriptorSet descriptorSets;

    public ConsumerRenderObject(VertexConsumer consumer, net.minecraft.client.render.VertexFormat.DrawMode drawMode, VertexFormat format, ShaderProgram program, UploadableImage image, Rosella rosella, Matrix4f projMatrix, Matrix4f viewMatrix) {
        ubo = new MinecraftUbo(rosella.getDevice(), rosella.getMemory());
        ubo.setMatrices(projMatrix, viewMatrix);
        this.consumer = consumer;
        this.drawMode = drawMode;
        this.format = format;
        this.shader = program;
        this.image = image;
    }

    //======================
    // Render Implementation
    //======================


    @Override
    public void load(@NotNull Rosella rosella) {
        switch (drawMode) {
            case TRIANGLES, QUADS -> {
                if (format != net.minecraft.client.render.VertexFormats.BLIT_SCREEN) {
                    material = Materials.TRIANGLES.build(shader, image, consumer.getFormat());
                }
            }

            case TRIANGLE_STRIP -> {
                if (format == net.minecraft.client.render.VertexFormats.POSITION) {
                    material = Materials.TRIANGLE_STRIP.build(shader, image, consumer.getFormat());
                }
            }

            case TRIANGLE_FAN -> material = Materials.TRIANGLE_FAN.build(shader, image, consumer.getFormat());

            default -> throw new RuntimeException("Unsupported Draw Mode:  " + drawMode);
        }
        descriptorSets = new DescriptorSet(material.shader.getRaw().getDescriptorPool());
        ubo.create(rosella.getRenderer().swapchain);
    }

    @Override
    public void free(@NotNull Memory memory, @NotNull Device device) {
        memory.freeBuffer(vertexBuffer);
        memory.freeBuffer(indexBuffer);
        ubo.free();
        descriptorSets.free(device);
    }

    @Override
    public void create(@NotNull Rosella rosella) {
        vertexBuffer = rosella.getMemory().createVertexBuffer(rosella, consumer);
        indexBuffer = rosella.getMemory().createIndexBuffer(rosella, indices);
        resize(rosella);
    }

    @Override
    public void resize(@NotNull Rosella rosella) {

        material.shader.getRaw().createDescriptorSets(rosella, this);
        if (!rosella.getPipelineManager().isValidPipeline(material.pipeline)) {
            material.pipeline = rosella.getPipelineManager().getPipeline(material, rosella.getRenderer(), rosella);
        }
    }

    @NotNull
    @Override
    public List<Integer> getIndices() {
        return indices;
    }

    @NotNull
    @Override
    public me.hydos.rosella.render.vertex.VertexConsumer render() {
        return consumer;
    }

    @NotNull
    @Override
    public DescriptorSet getDescriptorSet() {
        return descriptorSets;
    }

    @NotNull
    @Override
    public Material getMaterial() {
        return material;
    }

    @NotNull
    @Override
    public BufferInfo getVerticesBuffer() {
        return vertexBuffer;
    }

    @NotNull
    @Override
    public BufferInfo getIndicesBuffer() {
        return indexBuffer;
    }

    @NotNull
    @Override
    public Ubo getUbo() {
        return ubo;
    }

    @NotNull
    @Override
    public Matrix4f getTransformMatrix() {
        return transformationMatrix;
    }
}
