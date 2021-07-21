package me.hydos.blaze4d.api.shader;

import com.mojang.blaze3d.platform.Window;
import com.mojang.blaze3d.systems.RenderSystem;
import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.util.ConversionUtils;
import me.hydos.rosella.memory.BufferInfo;
import me.hydos.rosella.memory.Memory;
import me.hydos.rosella.render.descriptorsets.DescriptorSets;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.shader.ubo.Ubo;
import me.hydos.rosella.render.swapchain.Swapchain;
import net.minecraft.client.Minecraft;
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
    private final int totalSize;
    private final List<AddUboMemoryStep> steps;
    public DescriptorSets descSets;
    public List<BufferInfo> uboFrames = new ArrayList<>();
    public Matrix4f projectionMatrix;
    public Matrix4f viewTransformMatrix;
    public Vector3f chunkOffset;
    public com.mojang.math.Vector3f shaderLightDirections0;
    public com.mojang.math.Vector3f shaderLightDirections1;
    private int size;

    public MinecraftUbo(@NotNull Memory memory, Material material, List<AddUboMemoryStep> steps, int size) {
        this.memory = memory;
        this.descSets = new DescriptorSets(material.getShader().getRaw().getDescriptorPool());
        this.totalSize = size;
        this.steps = steps;
    }

    @Override
    public void create(Swapchain swapChain) {
        try (MemoryStack stack = MemoryStack.stackPush()) {
            this.free();
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
        return totalSize;
    }

    @Override
    public void update(int currentImg, @NotNull Swapchain swapChain) {
        if (uboFrames.size() == 0) {
            create(swapChain); //TODO: CONCERN. why did i write this
        }

        try (MemoryStack stack = MemoryStack.stackPush()) {
            PointerBuffer data = stack.mallocPointer(1);
            memory.map(uboFrames.get(currentImg).allocation(), false, data);
            ByteBuffer buffer = data.getByteBuffer(0, getSize());
            beginUboWrite();
            steps.forEach(addUboMemoryStep -> addUboMemoryStep.addUboMemoryStep(this, buffer));

//            memory.unmap(uboFrames.get(currentImg).allocation());
        }
    }

    public void addLightDirections1(ByteBuffer buffer) {
        putVec3f(shaderLightDirections1, buffer);
    }

    public void addLightDirections0(ByteBuffer buffer) {
        putVec3f(shaderLightDirections0, buffer);
    }

    public void addChunkOffset(ByteBuffer buffer) {
        putVec3f(chunkOffset, buffer);
    }

    public void addLineWidth(ByteBuffer buffer) {
        putFloat(RenderSystem.getShaderLineWidth(), buffer);
    }

    public void addScreenSize(ByteBuffer buffer) {
        Window window = Minecraft.getInstance().getWindow();
        putVec2i(window.getWidth(), window.getHeight(), buffer);
    }

    public void addGameTime(ByteBuffer buffer) {
        putFloat(RenderSystem.getShaderGameTime(), buffer);
    }

    public void addTextureMatrix(ByteBuffer buffer) {
        putMat4(ConversionUtils.mcToJomlMatrix(RenderSystem.getTextureMatrix()), buffer);
    }

    public void addFogColor(ByteBuffer buffer) {
        putVec4f(RenderSystem.getShaderFogColor(), buffer);
    }

    public void addFogEnd(ByteBuffer buffer) {
        putFloat(RenderSystem.getShaderFogEnd(), buffer);
    }

    public void addFogStart(ByteBuffer buffer) {
        putFloat(RenderSystem.getShaderFogStart(), buffer);
    }

    public void addShaderColor(ByteBuffer buffer) {
        putVec4f(RenderSystem.getShaderColor(), buffer);
    }

    public void addProjectionMatrix(ByteBuffer buffer) {
        putMat4(projectionMatrix, buffer);
    }

    public void addViewTransformMatrix(ByteBuffer buffer) {
        putMat4(viewTransformMatrix, buffer);
    }

    public static void addEndPortalLayers(MinecraftUbo minecraftUbo, ByteBuffer buffer) {
        buffer.putInt(15); // Taken from the end portal json         { "name": "EndPortalLayers", "type": "int", "count": 1, "values": [ 15 ] }
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

    protected void putVec3f(com.mojang.math.Vector3f vec3, ByteBuffer buffer) {
        putFloat(vec3.x(), buffer);
        putFloat(vec3.y(), buffer);
        putFloat(vec3.z(), buffer);
    }

    private void beginUboWrite() {
        size = 0;
    }

    public void setUniforms(Matrix4f projectionMatrix, Matrix4f viewTransformMatrix, Vector3f chunkOffset, com.mojang.math.Vector3f shaderLightDirections0, com.mojang.math.Vector3f shaderLightDirections1) {
        this.projectionMatrix = projectionMatrix;
        this.viewTransformMatrix = viewTransformMatrix;
        this.chunkOffset = chunkOffset;
        this.shaderLightDirections0 = shaderLightDirections0;
        this.shaderLightDirections1 = shaderLightDirections1;
    }

    @Override
    public void free() {
        for (BufferInfo uboImg : uboFrames) {
            uboImg.free(Blaze4D.rosella.common.device, memory);
        }
        uboFrames.clear();
    }

    @NotNull
    @Override
    public List<BufferInfo> getUniformBuffers() {
        return uboFrames;
    }

    @NotNull
    @Override
    public DescriptorSets getDescriptors() {
        return descSets;
    }

    @Override
    public void setDescriptors(@NotNull DescriptorSets descriptorSets) {
        this.descSets = descriptorSets;
    }

    public interface AddUboMemoryStep {
        void addUboMemoryStep(MinecraftUbo ubo, ByteBuffer buffer);
    }
}
