package me.hydos.blaze4d.api.shader;

import me.hydos.blaze4d.Blaze4D;
import me.hydos.rosella.memory.BufferInfo;
import me.hydos.rosella.memory.Memory;
import me.hydos.rosella.render.descriptorsets.DescriptorSets;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.shader.ubo.Ubo;
import me.hydos.rosella.render.swapchain.Swapchain;
import org.jetbrains.annotations.NotNull;
import org.lwjgl.PointerBuffer;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.system.MemoryUtil;
import org.lwjgl.util.vma.Vma;
import org.lwjgl.vulkan.VK10;

import java.nio.ByteBuffer;
import java.nio.LongBuffer;
import java.util.ArrayList;
import java.util.List;

public class MinecraftUbo extends Ubo {
    private final Memory memory;
    private final int totalSize;
    public DescriptorSets descSets;
    public List<BufferInfo> uboFrames = new ArrayList<>();
    private int size;
    private PointerBuffer pLocation;
    private ByteBuffer data;

    public MinecraftUbo(Memory memory, Material material, ByteBuffer shaderUbo) {
        this.memory = memory;
        this.descSets = new DescriptorSets(material.getShaderProgram().getRaw().getDescriptorPool());
        this.totalSize = shaderUbo.capacity();
        this.data = shaderUbo;
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

        if (pLocation == null) {
            pLocation = MemoryUtil.memAllocPointer(1);
            memory.map(uboFrames.get(currentImg).allocation(), false, pLocation);
        }

        beginUboWrite();
        ByteBuffer mainBuffer = pLocation.getByteBuffer(0, getSize());
        MemoryUtil.memCopy(data, mainBuffer);
    }

    private void beginUboWrite() {
        size = 0;
    }

    @Override
    public void free() {
        for (BufferInfo uboImg : uboFrames) {
            uboImg.free(Blaze4D.rosella.common.device, memory);
//            memory.unmap(uboImg.allocation());
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
}
