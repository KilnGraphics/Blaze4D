package me.hydos.blaze4d.mixin;

import com.mojang.datafixers.util.Pair;
import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.Materials;
import me.hydos.blaze4d.api.VertexResult;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.model.Renderable;
import me.hydos.rosella.render.model.Vertex;
import me.hydos.rosella.render.renderer.Renderer;
import me.hydos.rosella.render.shader.ubo.BasicUbo;
import me.hydos.rosella.render.shader.ubo.Ubo;
import me.hydos.rosella.render.util.memory.Memory;
import net.minecraft.client.gl.VertexBuffer;
import net.minecraft.client.render.*;
import net.minecraft.util.math.Vec3f;
import org.jetbrains.annotations.NotNull;
import org.joml.Matrix4f;
import org.joml.Vector2f;
import org.joml.Vector3f;
import org.lwjgl.util.vma.Vma;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.nio.ByteBuffer;
import java.nio.ByteOrder;
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

            int index = 0;
            for (int i = 0; i <= draw.getVertexCount() - 1; i++) {
                if (index >= byteBuffer.limit()) {
                    System.err.println("Managed to exceed byte buff limit somehow. this shouldn't happen! (I was: " + i + " and should of maxed out at " + draw.getVertexCount() + ")");
                    break;
                }
                VertexResult result = readVertex(byteBuffer, elementFormat);
                index += result.index();
                vertices.add(result.vertex());
                indices.add(indices.size());
                index += 12;
            }

//            System.out.println("================");
//            System.out.println(format);
//            System.out.println(max + " Bytes Total");
//            System.out.println(format.getVertexSize() + " Bytes Per Vertex");
//            System.out.println(draw.getVertexCount() + " Total Vertices");
//            System.out.println("================");

            if (!this.usesTexture) {
                byteBuffer.limit(draw.getDrawStart());
//                RenderSystem.glBufferData(34963, byteBuffer, 35044);
                Blaze4D.window.queue(() -> {
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

    private VertexResult readVertex(ByteBuffer vertBuf, VertexFormat format) {
        float x = 0, y = 0, z = 0; // Position
        float u = 0, v = 0; // UV
        int colorR = 0, colorG = 255, colorB = 0, colorA = 255; // Color
        int normalX = 0, normalY = 0, normalZ = 0; // Normal
        byte lighting = 0; // Padding

        for (VertexFormatElement element : format.getElements()) {
            switch (element.getType()) {
                case POSITION -> {
                    x = vertBuf.getFloat();
                    y = vertBuf.getFloat();
                    z = vertBuf.getFloat();
                }

                case COLOR -> {
                    colorR = getUnsignedByte(vertBuf);
                    colorG = getUnsignedByte(vertBuf);
                    colorB = getUnsignedByte(vertBuf);
                    colorA = getUnsignedByte(vertBuf);
                }

                case UV -> {
                    u = vertBuf.getFloat();
                    v = vertBuf.getFloat();
                }
                
                case NORMAL -> {
                    normalX = vertBuf.get();
                    normalY = vertBuf.get();
                    normalZ = vertBuf.get();
                }

                case PADDING -> {
                    lighting = vertBuf.get();
                }

                default -> System.out.println("Unknown Type: " + element.getType().getName());
            }
        }

        Vertex vertex = new Vertex(
                new Vector3f(x, y, z),
                new Vector3f(colorR, colorG, colorB),
                new Vector2f(u, v)
        );
        return new VertexResult(0, vertex);
    }


    /**
     * Read an unsigned byte from a buffer
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
        material = Materials.SOLID_COLOR;
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
