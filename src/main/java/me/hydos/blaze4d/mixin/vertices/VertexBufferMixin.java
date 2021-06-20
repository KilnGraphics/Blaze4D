package me.hydos.blaze4d.mixin.vertices;

import com.mojang.datafixers.util.Pair;
import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.Materials;
import me.hydos.blaze4d.api.shader.MinecraftUbo;
import me.hydos.blaze4d.api.vertex.RenderableConsumer;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.model.Renderable;
import me.hydos.rosella.render.renderer.Renderer;
import me.hydos.rosella.render.shader.ubo.Ubo;
import me.hydos.rosella.render.util.memory.Memory;
import me.hydos.rosella.render.vertex.VertexConsumer;
import net.minecraft.client.gl.VertexBuffer;
import net.minecraft.client.render.BufferBuilder;
import net.minecraft.client.render.BufferRenderer;
import net.minecraft.client.render.VertexFormat;
import org.jetbrains.annotations.NotNull;
import org.joml.Matrix4f;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.nio.ByteBuffer;
import java.util.ArrayList;
import java.util.List;

@Mixin(VertexBuffer.class)
public class VertexBufferMixin implements Renderable {

    @Shadow
    private int vertexCount;
    @Shadow
    private VertexFormat.IntType vertexFormat;
    @Shadow
    private VertexFormat elementFormat;
    @Shadow
    private VertexFormat.DrawMode drawMode;
    @Shadow
    private boolean usesTexture;

    public VertexConsumer consumer;
    public List<Integer> indices = new ArrayList<>();
    public Long vertexBuffer = 0L;
    public Long indexBuffer = 0L;

    public List<Long> descSets = new ArrayList<>();
    public Matrix4f transformationMatrix = new Matrix4f().identity();
    public Ubo ubo;
    public Material material;
    private RenderableConsumer builderStorage;

    @Inject(method = "uploadInternal", at = @At("HEAD"), cancellable = true)
    private void sendToRosella(BufferBuilder builder, CallbackInfo ci) {
        Pair<BufferBuilder.DrawArrayParameters, ByteBuffer> pair = builder.popData();
        BufferRenderer.unbindAll();
        BufferBuilder.DrawArrayParameters draw = pair.getFirst();
        ByteBuffer buffer = pair.getSecond();
        VertexFormat format = draw.getVertexFormat();

        this.vertexCount = draw.getVertexCount();
        this.vertexFormat = draw.getElementFormat();
        this.elementFormat = format;
        this.drawMode = draw.getMode();
        this.usesTexture = draw.isTextured();
        this.indices.clear();

        if (!draw.isCameraOffset()) {
            if (builder instanceof RenderableConsumer vertexStorage) {
                this.consumer = vertexStorage.getConsumer();
                this.builderStorage = vertexStorage;
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
            } else {
                throw new RuntimeException("Builder Cannot be cast to Blaze4dVertexStorage");
            }
        } else {
            throw new RuntimeException("Not Handled");
        }

        buffer.limit(draw.getDrawStart());
        buffer.position(0);
        Blaze4D.window.queue(() -> {
            Renderable remove = Blaze4D.rosella.getRenderObjects().remove(toString());
            if (remove != null) {
                remove.free(Blaze4D.rosella.getMemory());
            }

            if (consumer.getVertexCount() != 0) {
                Blaze4D.rosella.addRenderObject(this, toString());
            }
            Blaze4D.rosella.getRenderer().rebuildCommandBuffers(Blaze4D.rosella.getRenderer().renderPass, Blaze4D.rosella);
        });
        ci.cancel();
    }

    @Override
    public void load(@NotNull Rosella rosella) {
        switch (drawMode) {
            case TRIANGLES, QUADS -> {
                if (elementFormat != net.minecraft.client.render.VertexFormats.BLIT_SCREEN) {
                    material = Materials.TRIANGLES.build(builderStorage.getShader(), builderStorage.getImage(), consumer.getFormat());
                }
            }

            case TRIANGLE_STRIP -> {
                if (elementFormat == net.minecraft.client.render.VertexFormats.POSITION) {
                    material = Materials.TRIANGLE_STRIP.build(builderStorage.getShader(), builderStorage.getImage(), consumer.getFormat());
                }
            }

            case TRIANGLE_FAN -> material = Materials.TRIANGLE_FAN.build(builderStorage.getShader(), builderStorage.getImage(), consumer.getFormat());

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
    public List<Long> getDescriptorSets() {
        return descSets;
    }

    @Override
    public void setDescriptorSets(@NotNull List<Long> descSets) {
        this.descSets = descSets;
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

    @NotNull
    @Override
    public VertexConsumer render() {
        return consumer;
    }
}
