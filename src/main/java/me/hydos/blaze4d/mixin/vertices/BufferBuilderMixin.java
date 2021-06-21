package me.hydos.blaze4d.mixin.vertices;

import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.Materials;
import me.hydos.blaze4d.api.VkRenderSystem;
import me.hydos.blaze4d.api.shader.MinecraftUbo;
import me.hydos.blaze4d.api.vertex.RenderableConsumer;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.model.Renderable;
import me.hydos.rosella.render.renderer.Renderer;
import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.shader.ubo.Ubo;
import me.hydos.rosella.render.texture.UploadableImage;
import me.hydos.rosella.render.util.memory.Memory;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.render.*;
import org.jetbrains.annotations.NotNull;
import org.joml.Matrix4f;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.util.ArrayList;
import java.util.List;

@Mixin(BufferBuilder.class)
public abstract class BufferBuilderMixin extends FixedColorVertexConsumer implements RenderableConsumer, Renderable {

    @Shadow
    private VertexFormat format;

    @Shadow
    private VertexFormat.DrawMode drawMode;

    private me.hydos.rosella.render.vertex.BufferVertexConsumer consumer;
    private ShaderProgram shader;
    private long totalPos; // Used for caching/speeding up rendering
    private long prevTotalPos; // Used for caching/speeding up rendering

    // Renderable Fields
    private final Matrix4f transformationMatrix = new Matrix4f();
    private MinecraftUbo ubo;
    public Material material;
    public List<Integer> indices = new ArrayList<>();
    public Long vertexBuffer = 0L;
    public Long indexBuffer = 0L;
    public List<Long> descriptorSets = new ArrayList<>();

    @Inject(method = "begin", at = @At("HEAD"))
    private void setupConsumer(VertexFormat.DrawMode drawMode, VertexFormat format, CallbackInfo ci) {
        this.shader = VkRenderSystem.activeShader;

        if (format == VertexFormats.POSITION) {
            consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(me.hydos.rosella.render.vertex.VertexFormats.Companion.getPOSITION());
        } else if (format == VertexFormats.POSITION_COLOR) {
            consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(me.hydos.rosella.render.vertex.VertexFormats.Companion.getPOSITION_COLOR4());
        } else if (format == VertexFormats.POSITION_COLOR_TEXTURE) {
            consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(me.hydos.rosella.render.vertex.VertexFormats.Companion.getPOSITION_COLOR4_UV());
        } else if (format == VertexFormats.POSITION_TEXTURE) {
            consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(me.hydos.rosella.render.vertex.VertexFormats.Companion.getPOSITION_UV());
        } else if (format == VertexFormats.POSITION_TEXTURE_COLOR) {
            consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(me.hydos.rosella.render.vertex.VertexFormats.Companion.getPOSITION_UV_COLOR4());
        } else if (format == VertexFormats.LINES) {
            consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(me.hydos.rosella.render.vertex.VertexFormats.Companion.getPOSITION_COLOR_NORMAL_PADDING());
        } else if (format == VertexFormats.POSITION_COLOR_TEXTURE_LIGHT){
            consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(me.hydos.rosella.render.vertex.VertexFormats.Companion.getPOSITION_COLOR4_UV_LIGHT());
        } else {
            // Check if its text
            List<VertexFormatElement> elements = format.getElements();
            if (elements.size() == 4 && elements.get(0) == VertexFormats.POSITION_ELEMENT && elements.get(1) == VertexFormats.COLOR_ELEMENT && elements.get(2) == VertexFormats.TEXTURE_0_ELEMENT && elements.get(3).getByteLength() == 4) {
                consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(me.hydos.rosella.render.vertex.VertexFormats.Companion.getPOSITION_COLOR4_UV0_UV());
            } else {
                throw new RuntimeException("Format not implemented: " + format);
            }
        }
    }

    @Inject(method = "clear", at = @At("HEAD"))
    private void doCaching(CallbackInfo ci) {
        consumer.clear();
        indices.clear();

        prevTotalPos = totalPos;
        totalPos = 0;
    }

    @Override
    public VertexConsumer vertex(double x, double y, double z) {
        consumer.pos((float) x, (float) y, (float) z);
        this.totalPos += x + y + z;
        return this;
    }

    @Override
    public VertexConsumer color(int red, int green, int blue, int alpha) {
        consumer.color((byte) red, (byte) green, (byte) blue, (byte) alpha);
        return this;
    }

    @Override
    public VertexConsumer texture(float u, float v) {
        consumer.uv(u, v);
        return this;
    }

    @Override
    public VertexConsumer light(int light) {
        consumer.light(light);
        return this;
    }

    @Override
    public VertexConsumer overlay(int u, int v) {
        consumer.uv((short) u, (short) v);
        return this;
    }

    @Override
    public void next() {
        consumer.nextVertex();
    }

    @Override
    public me.hydos.rosella.render.vertex.BufferVertexConsumer getConsumer() {
        return consumer;
    }

    @Override
    public ShaderProgram getShader() {
        return shader;
    }

    @Override
    public UploadableImage getImage() {
        UploadableImage image = (UploadableImage) MinecraftClient.getInstance().getTextureManager().getTexture(VkRenderSystem.boundTexture);
        if (image == null) {
            throw new RuntimeException("Image is Null");
        }
        return image;
    }

    @Override
    public void draw() {
        if(prevTotalPos != totalPos) {
            if (indices.size() == 0) {
                for (int i = 0; i < consumer.getVertexCount(); i++) {
                    indices.add(i);
                }

                if (drawMode == VertexFormat.DrawMode.QUADS) {
                    // Convert Quads to Triangle Strips
                    //  0, 1, 2
                    //  0, 2, 3
                    //        v0_________________v1
                    //         / \               /
                    //        /     \           /
                    //       /         \       /
                    //      /             \   /
                    //    v2-----------------v3

                    indices.clear();
                    for (int i = 0; i < consumer.getVertexCount(); i += 4) {
                        indices.add(i);
                        indices.add(1 + i);
                        indices.add(2 + i);

                        indices.add(2 + i);
                        indices.add(3 + i);
                        indices.add(i);
                    }
                }
            }

            Renderable remove = Blaze4D.rosella.getRenderObjects().remove(toString());
            if (remove != null) {
                remove.free(Blaze4D.rosella.getMemory());
            }

            if (consumer.getVertexCount() != 0) {
                Blaze4D.rosella.addRenderObject(this, toString());
                Blaze4D.rosella.getRenderer().rebuildCommandBuffers(Blaze4D.rosella.getRenderer().renderPass, Blaze4D.rosella);
            }
        }
    }

    //======================
    // Render Implementation
    //======================


    @Override
    public void load(@NotNull Rosella rosella) {
        switch (drawMode) {
            case TRIANGLES, QUADS -> {
                if (format != net.minecraft.client.render.VertexFormats.BLIT_SCREEN) {
                    material = Materials.TRIANGLES.build(getShader(), getImage(), consumer.getFormat());
                }
            }

            case TRIANGLE_STRIP -> {
                if (format == net.minecraft.client.render.VertexFormats.POSITION) {
                    material = Materials.TRIANGLE_STRIP.build(getShader(), getImage(), consumer.getFormat());
                }
            }

            case TRIANGLE_FAN -> material = Materials.TRIANGLE_FAN.build(getShader(), getImage(), consumer.getFormat());

            default -> throw new RuntimeException("Unsupported Draw Mode:  " + drawMode);
        }
        ubo = new MinecraftUbo(rosella.getDevice(), rosella.getMemory());
        ubo.create(rosella.getRenderer().swapChain);
    }

    @Override
    public void free(@NotNull Memory memory) {
//        Vma.vmaFreeMemory(memory.getAllocator(), vertexBuffer);
//        Vma.vmaFreeMemory(memory.getAllocator(), indexBuffer);
        ubo.free();
    }

    @Override
    public void create(@NotNull Rosella rosella) {
        vertexBuffer = rosella.getMemory().createVertexBuffer(rosella, consumer);
        indexBuffer = rosella.getMemory().createIndexBuffer(rosella, indices);
        resize(rosella.getRenderer());
    }

    @Override
    public void resize(@NotNull Renderer renderer) {
        material.shader.getRaw().createDescriptorSets(renderer.swapChain, this);
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
    public List<Long> getDescriptorSets() {
        return descriptorSets;
    }

    @Override
    public void setDescriptorSets(@NotNull List<Long> descriptorSets) {
        this.descriptorSets = descriptorSets;
    }

    @NotNull
    @Override
    public Material getMaterial() {
        return material;
    }

    @Override
    public long getVerticesBuffer() {
        return vertexBuffer;
    }

    @Override
    public long getIndicesBuffer() {
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
