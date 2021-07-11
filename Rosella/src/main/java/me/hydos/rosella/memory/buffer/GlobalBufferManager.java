package me.hydos.rosella.memory.buffer;

import it.unimi.dsi.fastutil.objects.Object2IntMap;
import it.unimi.dsi.fastutil.objects.Object2IntOpenHashMap;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.memory.BufferInfo;
import me.hydos.rosella.memory.Memory;
import me.hydos.rosella.render.info.RenderInfo;
import me.hydos.rosella.render.renderer.Renderer;
import me.hydos.rosella.render.vertex.BufferProvider;
import me.hydos.rosella.vkobjects.VkCommon;
import org.lwjgl.system.MemoryStack;

import java.nio.ByteBuffer;
import java.nio.LongBuffer;
import java.util.HashSet;
import java.util.Set;

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

    public final Object2IntMap<RenderInfo> indicesOffsetMap = new Object2IntOpenHashMap<>();
    public final Object2IntMap<RenderInfo> vertexOffsetMap = new Object2IntOpenHashMap<>();

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

                int indexOffset = 0;
                for (RenderInfo renderInfo : renderList) {
                    indicesOffsetMap.put(renderInfo, indexOffset);
                    for (int index : renderInfo.indices) {
                        dst.putInt(index);
                    }
                    indexOffset += renderInfo.indices.size();
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
            totalSize += info.bufferProvider.getVertexSize() * info.bufferProvider.getVertexCount();
        }

        try (MemoryStack stack = stackPush()) {
            LongBuffer pBuffer = stack.mallocLong(1);
            int finalTotalSize = totalSize;

            BufferInfo stagingBuffer = memory.createStagingBuf(finalTotalSize, pBuffer, stack, data -> {
                ByteBuffer dst = data.getByteBuffer(0, finalTotalSize);
                int vertexOffset = 0;
                int bufferOffset = 0;
                for (RenderInfo renderInfo : renderList) {
                    vertexOffsetMap.put(renderInfo, vertexOffset);
                    for(BufferProvider.PositionedBuffer src : renderInfo.bufferProvider.getBuffers()) {
                        dst.put(bufferOffset + src.dstPos(), src.buffer(), src.srcPos(), src.length());
                    }
                    vertexOffset += renderInfo.bufferProvider.getVertexCount();
                    bufferOffset += renderInfo.bufferProvider.getVertexSize() * renderInfo.bufferProvider.getVertexCount();
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
