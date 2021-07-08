package me.hydos.rosella.memory.buffer;

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
import java.util.ArrayList;
import java.util.List;
import java.util.Set;
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

    public GlobalBufferManager(Rosella rosella) {
        this.memory = rosella.common.memory;
        this.common = rosella.common;
        this.renderer = rosella.renderer;
    }

    /**
     * Creates a index buffer based on the RenderInfo parsed in
     *
     * @param renderList the list of RenderInfo objects
     * @return a index buffer
     */
    public BufferInfo createIndexBuffer(Set<RenderInfo> renderList) {
        int totalSize = 0;
        for (RenderInfo info : renderList) {
            totalSize += info.getIndicesSize() * Integer.SIZE;
        }

        try (MemoryStack stack = stackPush()) {
            LongBuffer pBuffer = stack.mallocLong(1);
            int finalTotalSize = totalSize;

            BufferInfo stagingBuffer = memory.createStagingBuf(finalTotalSize, pBuffer, stack, data -> {
                ByteBuffer dst = data.getByteBuffer(0, finalTotalSize);

                Memory.memcpy(dst, toBigIndexList(renderList));
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
     * Converts all indices for all objects into a single list which can be used with a big vertex buffer
     *
     * @param renderList the render objects
     * @return a bit list of indices
     */
    private List<Integer> toBigIndexList(Set<RenderInfo> renderList) {
        List<Integer> bigNutIndices = new ArrayList<>(); // I'm sorry its just too funny

        RenderInfo lastRenderInfo = null;
        for (RenderInfo renderInfo : renderList) {
            int indexOffset = 0;

            if (lastRenderInfo != null) {
                BufferVertexConsumer lastConsumer = (BufferVertexConsumer) lastRenderInfo.consumer;
                indexOffset = lastConsumer.getBufferConsumerList().size() / lastConsumer.getFormat().getElements().size();
            }

            for (Integer index : renderInfo.indices) {
                bigNutIndices.add(indexOffset + index);
            }

            lastRenderInfo = renderInfo;
        }
        return bigNutIndices;
    }

    /**
     * Creates a vertex buffer based on the RenderInfo parsed in
     *
     * @param renderList the list of RenderInfo objects
     * @return a vertex buffer
     */
    public BufferInfo createVertexBuffer(Set<RenderInfo> renderList) {
        int totalSize = 0;
        for (RenderInfo info : renderList) {
            totalSize += info.consumer.getVertexSize() * info.consumer.getVertexCount();
        }

        try (MemoryStack stack = stackPush()) {
            LongBuffer pBuffer = stack.mallocLong(1);
            int finalTotalSize = totalSize;

            BufferInfo stagingBuffer = memory.createStagingBuf(finalTotalSize, pBuffer, stack, data -> {
                ByteBuffer dst = data.getByteBuffer(0, finalTotalSize);
                for (RenderInfo renderInfo : renderList) {
                    for (Consumer<ByteBuffer> bufConsumer : ((BufferVertexConsumer) renderInfo.consumer).getBufferConsumerList()) {
                        bufConsumer.accept(dst);
                    }
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
