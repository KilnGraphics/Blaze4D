package me.hydos.blaze4d.api.shader;

import com.mojang.blaze3d.systems.RenderSystem;
import me.hydos.rosella.render.descriptorsets.DescriptorSet;
import me.hydos.rosella.render.device.Device;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.shader.ubo.Ubo;
import me.hydos.rosella.render.swapchain.Swapchain;
import me.hydos.rosella.render.util.memory.BufferInfo;
import me.hydos.rosella.render.util.memory.Memory;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.util.Window;
import net.minecraft.util.math.Vec3f;
import org.jetbrains.annotations.NotNull;
import org.joml.Matrix4f;
import org.joml.Vector3f;
import org.lwjgl.PointerBuffer;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.util.vma.Vma;
import org.lwjgl.vulkan.VK10;

import java.nio.ByteBuffer;
import java.nio.LongBuffer;
import java.util.ArrayList;
import java.util.List;

public class MinecraftUbo extends Ubo {

    private final Memory memory;
    private int size;
    public DescriptorSet descSets;
    public List<BufferInfo> uboFrames = new ArrayList<>();

    public Matrix4f projectionMatrix;
    public Matrix4f viewTransformMatrix;
    public Vector3f chunkOffset;
    public Vec3f shaderLightDirections0;
    public Vec3f shaderLightDirections1;

    public MinecraftUbo(@NotNull Device device, @NotNull Memory memory, Material material) {
        this.memory = memory;
        this.descSets = new DescriptorSet(material.shader.getRaw().getDescriptorPool());
    }

    @Override
    public void create(Swapchain swapChain) {
        try (MemoryStack stack = MemoryStack.stackPush()) {
            uboFrames = new ArrayList<>(swapChain.getSwapChainImages().size());
            for (int i = 0; i < swapChain.getSwapChainImages().size(); i++) {
                LongBuffer pBuffer = stack.mallocLong(1);
                uboFrames.add(
                        memory.createBuffer(
                                getSize(),
                                VK10.VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT,
                                Vma.VMA_MEMORY_USAGE_CPU_ONLY,
                                pBuffer
                        )
                );
            }
        }
    }

    @Override
    public int getSize() {
        return 296;
    }

    @Override
    public void update(int currentImg, @NotNull Swapchain swapChain) {
        if (uboFrames.size() == 0) {
            create(swapChain); //TODO: CONCERN. why did i write this
        }

        try (MemoryStack stack = MemoryStack.stackPush()) {
            PointerBuffer data = stack.mallocPointer(1);
            memory.map(uboFrames.get(currentImg).getAllocation(), false, data);
            ByteBuffer buffer = data.getByteBuffer(0, getSize());
            Window window = MinecraftClient.getInstance().getWindow();

            beginUboWrite();
            putMat4(viewTransformMatrix, buffer); // ModelViewMat
            putMat4(projectionMatrix, buffer); // ProjectionMat
            putVec4f(RenderSystem.getShaderColor(), buffer); // ColorModulator
            putFloat(RenderSystem.getShaderFogStart(), buffer); // FogStart
            putFloat(RenderSystem.getShaderFogEnd(), buffer); // FogEnd
            putVec4f(RenderSystem.getShaderFogColor(), buffer); // FogColor
            putMat4(toJoml(RenderSystem.getTextureMatrix()), buffer); // TextureMat
            putFloat(RenderSystem.getShaderGameTime(), buffer); // GameTime
            putVec2i(window.getFramebufferWidth(), window.getFramebufferHeight(), buffer); // ScreenSize
            putFloat(RenderSystem.getShaderLineWidth(), buffer); // LineWidth
            putVec3f(chunkOffset, buffer); // ChunkOffset
            putVec3f(shaderLightDirections0, buffer); // Light0_Direction
            putVec3f(shaderLightDirections1, buffer); // Light1_Direction

            memory.unmap(uboFrames.get(currentImg).getAllocation());
        }
    }

    protected void putVec2i(int i1, int i2, ByteBuffer buffer) {
        putInt(i1, buffer);
        putInt(i2, buffer);
    }

    protected void putMat4(Matrix4f matrix4f, ByteBuffer buffer) {
        if (size == 0) {
            matrix4f.get(0, buffer);
        } else {
            matrix4f.get(size, buffer);
        }
        size += 16 * Float.BYTES;
    }

    protected void putFloat(float f, ByteBuffer buffer) {
        if (size == 0) {
            buffer.putFloat(f);
        } else {
            buffer.putFloat(size, f);
        }
        size += Float.BYTES;
    }

    protected void putInt(int i, ByteBuffer buffer) {
        if (size == 0) {
            buffer.putInt(i);
        } else {
            buffer.putInt(size, i);
        }
        size += Integer.BYTES;
    }

    protected void putVec4f(float[] vec4, ByteBuffer buffer) {
        putFloat(vec4[0], buffer);
        putFloat(vec4[1], buffer);
        putFloat(vec4[2], buffer);
        putFloat(vec4[3], buffer);
    }

    protected void putVec3f(Vector3f vec3, ByteBuffer buffer) {
        putFloat(vec3.x, buffer);
        putFloat(vec3.y, buffer);
        putFloat(vec3.z, buffer);
    }

    protected void putVec3f(Vec3f vec3, ByteBuffer buffer) {
        putFloat(vec3.getX(), buffer);
        putFloat(vec3.getY(), buffer);
        putFloat(vec3.getZ(), buffer);
    }

    private void beginUboWrite() {
        size = 0;
    }

    public void setUniforms(Matrix4f projectionMatrix, Matrix4f viewTransformMatrix, Vector3f chunkOffset, Vec3f shaderLightDirections0, Vec3f shaderLightDirections1) {
        this.projectionMatrix = projectionMatrix;
        this.viewTransformMatrix = viewTransformMatrix;
        this.chunkOffset = chunkOffset;
        this.shaderLightDirections0 = shaderLightDirections0;
        this.shaderLightDirections1 = shaderLightDirections1;
    }

    public static Matrix4f toJoml(net.minecraft.util.math.Matrix4f mcMatrix) {
        Matrix4f jomlMatrix = new Matrix4f();

        jomlMatrix.m00(mcMatrix.a00);
        jomlMatrix.m01(mcMatrix.a10);
        jomlMatrix.m02(mcMatrix.a20);
        jomlMatrix.m03(mcMatrix.a30);

        jomlMatrix.m10(mcMatrix.a01);
        jomlMatrix.m11(mcMatrix.a11);
        jomlMatrix.m12(mcMatrix.a21);
        jomlMatrix.m13(mcMatrix.a31);

        jomlMatrix.m20(mcMatrix.a02);
        jomlMatrix.m21(mcMatrix.a12);
        jomlMatrix.m22(mcMatrix.a22);
        jomlMatrix.m23(mcMatrix.a32);

        jomlMatrix.m30(mcMatrix.a03);
        jomlMatrix.m31(mcMatrix.a13);
        jomlMatrix.m32(mcMatrix.a23);
        jomlMatrix.m33(mcMatrix.a33);

        return jomlMatrix;
    }

    @Override
    public void free() {
        for (BufferInfo uboImg : uboFrames) {
            memory.freeBuffer(uboImg);
        }
    }

    @NotNull
    @Override
    public List<BufferInfo> getUniformBuffers() {
        return uboFrames;
    }

    @NotNull
    @Override
    public DescriptorSet getDescriptors() {
        return descSets;
    }

    @Override
    public void setDescriptors(@NotNull DescriptorSet descriptorSets) {
        this.descSets = descriptorSets;
    }
}
