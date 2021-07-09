package me.hydos.rosella.memory.buffer;

import it.unimi.dsi.fastutil.objects.Object2IntOpenHashMap;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.memory.BufferInfo;
import me.hydos.rosella.memory.Memory;
import me.hydos.rosella.render.info.RenderInfo;
import me.hydos.rosella.render.renderer.Renderer;
import me.hydos.rosella.render.vertex.BufferVertexConsumer;
import me.hydos.rosella.vkobjects.VkCommon;
import org.lwjgl.system.MemoryStack;

import java.nio.ByteBuffer;
import java.nio.LongBuffer;
import java.util.*;
import java.util.function.Consumer;

import static org.lwjgl.system.MemoryStack.stackPush;
import static org.lwjgl.util.vma.Vma.VMA_MEMORY_USAGE_CPU_TO_GPU;
import static org.lwjgl.vulkan.VK10.*;

/**
 * To do what all the cool kids do, we make 2 big nut buffers TM
 */
public class GlobalBufferManager {

    private final Memory memory;
    private final VkCommon common;
    private final Renderer renderer;

    private Set<RenderInfo> lastRenderObjects;

    public BufferInfo vertexBuffer;
    public BufferInfo indexBuffer;

    public final Map<RenderInfo, Integer> indicesOffsetMap = new Object2IntOpenHashMap<>();
    public final Map<RenderInfo, Integer> vertexOffsetMap = new Object2IntOpenHashMap<>();

    public GlobalBufferManager(Rosella rosella) {
        this.memory = rosella.common.memory;
        this.common = rosella.common;
        this.renderer = rosella.renderer;
    }

    public void nextFrame(Set<RenderInfo> renderObjects) {
        if (isFrameDifferent(lastRenderObjects, renderObjects)) {
            this.vertexBuffer = createVertexBuffer(renderObjects);
            this.indexBuffer = createIndexBuffer(renderObjects);
        }

        lastRenderObjects = new HashSet<>(renderObjects);
    }

    private boolean isFrameDifferent(Set<RenderInfo> lastRenderObjects, Set<RenderInfo> renderObjects) {
        return lastRenderObjects == null || !lastRenderObjects.equals(renderObjects);
    }

    /**
     * Creates a index buffer based on the RenderInfo parsed in
     *
     * @param renderList the list of RenderInfo objects
     * @return a index buffer
     */
    private BufferInfo createIndexBuffer(Set<RenderInfo> renderList) {
        int totalSize = 0;
        for (RenderInfo info : renderList) {
            totalSize += info.getIndicesSize() * Integer.BYTES;
        }

        try (MemoryStack stack = stackPush()) {
            LongBuffer pBuffer = stack.mallocLong(1);
            int finalTotalSize = totalSize;

            BufferInfo stagingBuffer = memory.createStagingBuf(finalTotalSize, pBuffer, stack, data -> {
                ByteBuffer dst = data.getByteBuffer(0, finalTotalSize);

                List<Integer> allIndices = new ArrayList<>();
                for (RenderInfo renderInfo : renderList) {
                    indicesOffsetMap.put(renderInfo, allIndices.size());
                    Memory.memcpy(dst, renderInfo.indices);
                    allIndices.addAll(renderInfo.indices);
                }
            });

            BufferInfo indexBuffer = memory.createBuffer(
                    finalTotalSize,
                    VK_BUFFER_USAGE_TRANSFER_DST_BIT | VK_BUFFER_USAGE_INDEX_BUFFER_BIT,
                    VMA_MEMORY_USAGE_CPU_TO_GPU,
                    pBuffer
            );

            long pIndexBuffer = pBuffer.get(0);
            memory.copyBuffer(stagingBuffer.buffer(),
                    pIndexBuffer,
                    finalTotalSize,
                    renderer,
                    common.device);
            stagingBuffer.free(common.device, memory);

            return indexBuffer;
        }
    }

    /**
     * Creates a vertex buffer based on the RenderInfo parsed in
     *
     * @param renderList the list of RenderInfo objects
     * @return a vertex buffer
     */
    private BufferInfo createVertexBuffer(Set<RenderInfo> renderList) {
        int totalSize = 0;
        for (RenderInfo info : renderList) {
            totalSize += info.consumer.getVertexSize() * info.consumer.getVertexCount();
        }

        try (MemoryStack stack = stackPush()) {
            LongBuffer pBuffer = stack.mallocLong(1);
            int finalTotalSize = totalSize;

            BufferInfo stagingBuffer = memory.createStagingBuf(finalTotalSize, pBuffer, stack, data -> {
                ByteBuffer dst = data.getByteBuffer(0, finalTotalSize);
                int vertexOffset = 0;
                for (RenderInfo renderInfo : renderList) {
                    vertexOffsetMap.put(renderInfo, vertexOffset);
                    for (Consumer<ByteBuffer> bufConsumer : ((BufferVertexConsumer) renderInfo.consumer).getBufferConsumerList()) {
                        bufConsumer.accept(dst);
                    }
                    vertexOffset += renderInfo.consumer.getVertexSize() * renderInfo.consumer.getVertexCount();
                }
            });

            BufferInfo vertexBuffer = memory.createBuffer(
                    finalTotalSize,
                    VK_BUFFER_USAGE_TRANSFER_DST_BIT | VK_BUFFER_USAGE_VERTEX_BUFFER_BIT,
                    VMA_MEMORY_USAGE_CPU_TO_GPU,
                    pBuffer
            );

            long pVertexBuffer = pBuffer.get(0);
            memory.copyBuffer(stagingBuffer.buffer(),
                    pVertexBuffer,
                    finalTotalSize,
                    renderer,
                    common.device);
            stagingBuffer.free(common.device, memory);

            return vertexBuffer;
        }
    }
}
