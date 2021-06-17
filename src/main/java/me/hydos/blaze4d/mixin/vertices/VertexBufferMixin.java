package me.hydos.blaze4d.mixin.vertices;

import com.mojang.datafixers.util.Pair;
import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.Materials;
import me.hydos.blaze4d.api.shader.MinecraftUbo;
import me.hydos.blaze4d.api.vertex.Blaze4dVertexStorage;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.model.Renderable;
import me.hydos.rosella.render.model.Vertex;
import me.hydos.rosella.render.renderer.Renderer;
import me.hydos.rosella.render.shader.ubo.Ubo;
import me.hydos.rosella.render.util.memory.Memory;
import net.minecraft.client.gl.VertexBuffer;
import net.minecraft.client.render.BufferBuilder;
import net.minecraft.client.render.BufferRenderer;
import net.minecraft.client.render.VertexFormat;
import org.jetbrains.annotations.NotNull;
import org.joml.Matrix4f;
import org.joml.Vector2f;
import org.joml.Vector3f;
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
    @Shadow
    private int vertexBufferId;

    public List<Vertex> vertices = new ArrayList<>();
    public List<Integer> indices = new ArrayList<>();
    public Long vertexBuffer = 0L;
    public Long indexBuffer = 0L;

    public List<Long> descSets = new ArrayList<>();
    public Matrix4f transformationMatrix = new Matrix4f();
    public Ubo ubo;
    public Material material;

    @Inject(method = "uploadInternal", at = @At("HEAD"), cancellable = true)
    private void sendToRosella(BufferBuilder builder, CallbackInfo ci) {
        Pair<BufferBuilder.DrawArrayParameters, ByteBuffer> pair = builder.popData();
        if (this.vertexBufferId != 0) {
            BufferRenderer.unbindAll();
            BufferBuilder.DrawArrayParameters draw = pair.getFirst();
            ByteBuffer buffer = pair.getSecond();
            VertexFormat format = draw.getVertexFormat();

            this.vertexCount = draw.getVertexCount();
            this.vertexFormat = draw.getElementFormat();
            this.elementFormat = format;
            this.drawMode = draw.getMode();
            this.usesTexture = draw.isTextured();
            this.vertices.clear();
            this.indices.clear();

            if (!draw.isCameraOffset()) {
                if (builder instanceof Blaze4dVertexStorage vertexStorage) {
                    for (Blaze4dVertexStorage.VertexData data : vertexStorage.getVertices()) {
                        Vertex vertex = new Vertex(
                                new Vector3f(data.x(), data.y(), data.z()),
                                new Vector3f(data.r() / 255f, data.g() / 255f, data.b() / 255f),
                                new Vector2f(0, 0)
                        );
                        vertices.add(vertex);
                        indices.add(indices.size());
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
                        for (int i = 0; i < vertices.size(); i += 4) {
                            indices.add(i);
                            indices.add(1 + i);
                            indices.add(2 + i);

                            indices.add(i);
                            indices.add(2 + i);
                            indices.add(3 + i);
                        }
                    }
                } else {
                    throw new RuntimeException("Builder Cannot be cast to Blaze4dVertexStorage");
                }
            }

            buffer.limit(draw.getDrawStart());
//            if (!this.usesTexture) {
//                // TODO: read vertices from here.
//                if (drawMode != VertexFormat.DrawMode.QUADS) {
//                    indices = readIndices(buffer);
//                }
//            }
            buffer.position(0);
            Blaze4D.window.queue(() -> {
                Renderable remove = Blaze4D.rosella.getRenderObjects().remove(toString());
                if (remove != null) {
                    remove.free(Blaze4D.rosella.getMemory());
                }

                if (this.vertices.size() != 0) {
                    Blaze4D.rosella.addRenderObject(this, toString());
                }
                Blaze4D.rosella.getRenderer().rebuildCommandBuffers(Blaze4D.rosella.getRenderer().renderPass, Blaze4D.rosella);
            });
        }
        ci.cancel();
    }

    @Override
    public void load(@NotNull Rosella rosella) {
        switch (drawMode) {
            case TRIANGLES, QUADS -> material = Materials.SOLID_COLOR_TRIANGLES;

            case TRIANGLE_STRIP -> material = Materials.SOLID_COLOR_TRIANGLE_STRIP;

            case TRIANGLE_FAN -> material = Materials.SOLID_COLOR_TRIANGLE_FAN;

            default -> throw new RuntimeException("FUCK " + drawMode);
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
        vertexBuffer = rosella.getMemory().createVertexBuffer(rosella, vertices);
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
    public List<Vertex> getVertices() {
        return vertices;
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
}
