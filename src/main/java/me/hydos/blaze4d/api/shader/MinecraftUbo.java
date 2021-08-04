package me.hydos.blaze4d.api.shader;

import it.unimi.dsi.fastutil.ints.Int2IntMap;
import it.unimi.dsi.fastutil.ints.Int2IntMaps;
import it.unimi.dsi.fastutil.ints.Int2IntOpenHashMap;
import me.hydos.blaze4d.Blaze4D;
import me.hydos.rosella.memory.BufferInfo;
import me.hydos.rosella.memory.Memory;
import me.hydos.rosella.render.descriptorsets.DescriptorSets;
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
    public static final Int2IntMap UNIFORM_OFFSETS;

    static {
        Int2IntMap map = new Int2IntOpenHashMap();
        map.put(0, Integer.BYTES);
        map.put(1, 2 * Integer.BYTES);
        map.put(2, 4 * Integer.BYTES);
        map.put(3, 4 * Integer.BYTES);
        map.put(4, Float.BYTES);
        map.put(5, 2 * Float.BYTES);
        map.put(6, 4 * Float.BYTES);
        map.put(7, 4 * Float.BYTES);
        map.put(10, 4 * 4 * Float.BYTES);
        // by default, the map will return 0. instead, let's make it explode the program.
        map.defaultReturnValue(Integer.MIN_VALUE);
        UNIFORM_OFFSETS = Int2IntMaps.unmodifiable(map);
    }

    private final Memory memory;
    private final int totalSize;
    public DescriptorSets descSets;
    public List<BufferInfo> uboFrames = new ArrayList<>();
//    private PointerBuffer pLocation;
    private ByteBuffer data; // TODO: when do we free this?

    public MinecraftUbo(Memory memory, long rawDescriptorPool, ByteBuffer shaderUbo) {
        this.memory = memory;
        this.descSets = new DescriptorSets(rawDescriptorPool);
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

//        if (pLocation == null) {
//            PointerBuffer pLocation = MemoryUtil.memAllocPointer(1);
//            memory.map(uboFrames.get(currentImg).allocation(), false, pLocation);
//        }
        try (MemoryStack stack = MemoryStack.stackPush()) {
            PointerBuffer pLocation = stack.mallocPointer(1);
            memory.map(uboFrames.get(currentImg).allocation(), false, pLocation);

            ByteBuffer mainBuffer = pLocation.getByteBuffer(0, getSize());
            MemoryUtil.memCopy(data, mainBuffer);
        }
    }

    @Override
    public void free() {
        for (BufferInfo uboImg : uboFrames) {
            uboImg.free(Blaze4D.rosella.common.device, memory);
            memory.unmap(uboImg.allocation());
        }
//        MemoryUtil.memFree(pLocation);
//        pLocation = null;
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
