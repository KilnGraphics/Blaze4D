package me.hydos.blaze4d.api.shader;

import it.unimi.dsi.fastutil.ints.Int2IntMap;
import it.unimi.dsi.fastutil.ints.Int2IntMaps;
import it.unimi.dsi.fastutil.ints.Int2IntOpenHashMap;
import it.unimi.dsi.fastutil.longs.Long2ObjectMap;
import it.unimi.dsi.fastutil.longs.Long2ObjectOpenHashMap;
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
    private boolean dirty;
    private DescriptorSets descSets;
    private final List<BufferInfo> uboFrames = new ArrayList<>();
    private final Long2ObjectMap<PointerBuffer> mappedAllocations = new Long2ObjectOpenHashMap<>();
    private ByteBuffer data;

    public MinecraftUbo(Memory memory, long rawDescriptorPool, ByteBuffer rawUboData) {
        this.memory = memory;
        this.descSets = new DescriptorSets(rawDescriptorPool);
        this.totalSize = rawUboData.capacity();
        this.data = rawUboData;
    }

    public void markDirty(ByteBuffer data) {
        dirty = true;
        this.data = data;
    }

    @Override
    public void create(Swapchain swapChain) {
        try (MemoryStack stack = MemoryStack.stackPush()) {
            this.freeUboFrames();
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
        if (uboFrames.isEmpty()) return;

        if (dirty) {
            PointerBuffer pLocation = mappedAllocations.computeIfAbsent(uboFrames.get(currentImg).allocation(), allocation -> {
                PointerBuffer newPointer = MemoryUtil.memAllocPointer(1);
                memory.map(allocation, true, newPointer);
                return newPointer;
            });

            ByteBuffer mainBuffer = pLocation.getByteBuffer(0, getSize());
            MemoryUtil.memCopy(data, mainBuffer);

            dirty = false;
        }
    }

    public void freeUboFrames() {
        for (BufferInfo uboImg : uboFrames) {
            uboImg.free(Blaze4D.rosella.common.device, memory);
            memory.unmap(uboImg.allocation());
        }

        for (PointerBuffer pointer : mappedAllocations.values()) {
            MemoryUtil.memFree(pointer);
        }

        mappedAllocations.clear();
        uboFrames.clear();
    }

    @Override
    public void free() {
        freeUboFrames();
        MemoryUtil.memFree(data);
        data = null;
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
