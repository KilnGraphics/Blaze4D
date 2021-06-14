package me.hydos.blaze4d.mixin;

import com.mojang.datafixers.util.Pair;
import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.Materials;
import me.hydos.blaze4d.api.vertex.VertexBuilder;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.model.Renderable;
import me.hydos.rosella.render.model.Vertex;
import me.hydos.rosella.render.renderer.Renderer;
import me.hydos.rosella.render.shader.ubo.BasicUbo;
import me.hydos.rosella.render.shader.ubo.Ubo;
import me.hydos.rosella.render.util.memory.Memory;
import net.minecraft.client.gl.VertexBuffer;
import net.minecraft.client.render.BufferBuilder;
import net.minecraft.client.render.BufferRenderer;
import net.minecraft.client.render.VertexFormat;
import net.minecraft.client.render.VertexFormatElement;
import org.jetbrains.annotations.NotNull;
import org.joml.Matrix4f;
import org.joml.Vector2f;
import org.joml.Vector3f;
import org.joml.Vector4f;
import org.lwjgl.util.vma.Vma;
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
    private VertexBuilder vertexBuilder;

    private static float cccc = 0;

    @Inject(method = "uploadInternal", at = @At("HEAD"), cancellable = true)
    private void sendToRosella(BufferBuilder buffer, CallbackInfo ci) {
        Pair<BufferBuilder.DrawArrayParameters, ByteBuffer> pair = buffer.popData();
        if (this.vertexBufferId != 0) {
            BufferRenderer.unbindAll();
            BufferBuilder.DrawArrayParameters draw = pair.getFirst();
            ByteBuffer byteBuffer = pair.getSecond();
            VertexFormat format = draw.getVertexFormat();

            this.vertexCount = draw.getVertexCount();
            this.vertexFormat = draw.getElementFormat();
            this.elementFormat = format;
            this.drawMode = draw.getMode();
            this.usesTexture = draw.isTextured();
            this.vertexBuilder = new VertexBuilder();

            for (int i = 0; i <= draw.getVertexCount() - 1; i++) {
                try {
                    List<Vertex> result = readVertex(byteBuffer, elementFormat, vertexBuilder);
                    vertices.addAll(result);
                    for (Vertex ignored : result) {
                        indices.add(indices.size());
                    }
                    vertexBuilder.next(byteBuffer);
                }catch (IndexOutOfBoundsException e) {
                    break;
                }
            }
            cccc++;

            if (true) {
                byteBuffer.limit(draw.getDrawStart());
                Blaze4D.window.queue(() -> {
                    Blaze4D.rosella.getRenderObjects().remove(toString());
                    Blaze4D.rosella.addRenderObject(this, toString());
                    Blaze4D.rosella.getRenderer().rebuildCommandBuffers(Blaze4D.rosella.getRenderer().renderPass, Blaze4D.rosella);
                });
            } else {
                byteBuffer.limit(draw.getDrawStart());
            }
            byteBuffer.position(0);
        }
        ci.cancel();
    }

    private List<Vertex> readVertex(ByteBuffer vertBuf, VertexFormat format, VertexBuilder vertexBuilder) {
        Vector3f position = new Vector3f(); // Position
        Vector2f uv = new Vector2f(); // UV
        Vector4f color = new Vector4f(); // Color
        Vector3f normal = new Vector3f(); // Normal
        int light = 0; // Padding?

        for (VertexFormatElement element : format.getElements()) {
            switch (element.getType()) {
                case POSITION -> position = vertexBuilder.vertex(vertBuf, element);
                case COLOR -> color = vertexBuilder.color(vertBuf, element);
                case NORMAL -> normal = vertexBuilder.normal(vertBuf, element);
                case UV -> uv = vertexBuilder.texture(vertBuf, element);
                case PADDING, GENERIC -> vertexBuilder.padding(vertBuf, element);

                default -> System.err.println("Unknown Type: " + element.getType().getName());
            }
        }

//        int bytesRead = vertexBuilder.index;
//        if (bytesRead != format.getVertexSize() ) {
//            System.err.println("================");
//            System.err.println("Vertex Format: " + format);
//            System.err.println("Vertex Format Elements: " + format.getElements());
//            System.err.println("An Underflow was Caught. (Was Meant to read " + format.getVertexSize() + " Bytes but actually read " + bytesRead + ")");
//            System.err.println("================");
//        }
        List<Vertex> newVertices = new ArrayList<>();

        newVertices.add(new Vertex(
                position,
                new Vector3f(0, cccc, cccc),
                uv
        ));

        return newVertices;
    }


    /**
     * Read an unsigned byte from a buffer
     *
     * @param buffer Buffer containing the bytes
     * @return The unsigned byte as an int
     */
    public int getUnsignedByte(ByteBuffer buffer) {
        return asUnsignedByte(buffer.get());
    }

    /**
     * @return the byte value converted to an unsigned int value
     */
    public int asUnsignedByte(byte b) {
        return b & 0xFF;
    }

    @Override
    public void load(@NotNull Rosella rosella) {
        material = drawMode == VertexFormat.DrawMode.TRIANGLE_STRIP ? Materials.SOLID_COLOR_TRIANGLE_STRIP : Materials.SOLID_COLOR_TRIANGLES;
        ubo = new BasicUbo(rosella.getDevice(), rosella.getMemory());
        ubo.create(rosella.getRenderer().swapChain);
    }

    @Override
    public void free(@NotNull Memory memory) {
        Vma.vmaFreeMemory(memory.getAllocator(), vertexBuffer);
        Vma.vmaFreeMemory(memory.getAllocator(), indexBuffer);
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
