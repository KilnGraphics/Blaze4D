package me.hydos.rosella.memory.buffer;

import com.google.common.hash.HashFunction;
import com.google.common.hash.Hashing;
import it.unimi.dsi.fastutil.ints.Int2ObjectMap;
import it.unimi.dsi.fastutil.ints.Int2ObjectOpenCustomHashMap;
import it.unimi.dsi.fastutil.ints.IntHash;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.memory.BufferInfo;
import me.hydos.rosella.memory.ManagedBuffer;
import me.hydos.rosella.memory.Memory;
import me.hydos.rosella.render.renderer.Renderer;
import me.hydos.rosella.vkobjects.VkCommon;
import org.lwjgl.system.MemoryStack;

import java.nio.ByteBuffer;
import java.nio.LongBuffer;
import java.util.concurrent.atomic.AtomicInteger;

import static org.lwjgl.system.MemoryStack.stackPush;
import static org.lwjgl.util.vma.Vma.VMA_MEMORY_USAGE_CPU_TO_GPU;
import static org.lwjgl.util.vma.Vma.VMA_MEMORY_USAGE_GPU_ONLY;
import static org.lwjgl.vulkan.VK10.*;

/**
 * To do what all the cool kids do, we make 2 big nut buffers TM
 */
public class GlobalBufferManager {

    private static final HashFunction BUFFER_HASH_FUNCTION = Hashing.murmur3_32();
    private static final IntHash.Strategy PREHASHED_STRATEGY = new IntHash.Strategy() {
        @Override
        public int hashCode(int e) {
            return e;
        }

        @Override
        public boolean equals(int a, int b) {
            return a == b;
        }
    };

    private final Memory memory;
    private final VkCommon common;
    private final Renderer renderer;

    private final Int2ObjectMap<BufferInfo> vertexHashToBufferMap = new Int2ObjectOpenCustomHashMap<>(PREHASHED_STRATEGY);
    private final Int2ObjectMap<AtomicInteger> vertexHashToInvocationsFrameMap = new Int2ObjectOpenCustomHashMap<>(PREHASHED_STRATEGY);

    private final Int2ObjectMap<BufferInfo> indexHashToBufferMap = new Int2ObjectOpenCustomHashMap<>(PREHASHED_STRATEGY);
    private final Int2ObjectMap<AtomicInteger> indexHashToInvocationsFrameMap = new Int2ObjectOpenCustomHashMap<>(PREHASHED_STRATEGY);

    public GlobalBufferManager(Rosella rosella) {
        this.memory = rosella.common.memory;
        this.common = rosella.common;
        this.renderer = rosella.renderer;
    }

    public void postDraw() {
        for (Int2ObjectMap.Entry<AtomicInteger> entry : vertexHashToInvocationsFrameMap.int2ObjectEntrySet()) {
            if (entry.getValue().getAcquire() < 1) {
                vertexHashToBufferMap.remove(entry.getIntKey()).free(common.device, memory);
            }
        }
        vertexHashToInvocationsFrameMap.clear();

        for (Int2ObjectMap.Entry<AtomicInteger> entry : indexHashToInvocationsFrameMap.int2ObjectEntrySet()) {
            if (entry.getValue().getAcquire() < 1) {
                indexHashToBufferMap.remove(entry.getIntKey()).free(common.device, memory);
            }
        }
        indexHashToInvocationsFrameMap.clear();
    }

    /**
     * Gets or creates an index buffer
     *
     * @param indexBytes The bytes representing the indices.
     *                   This buffer must have the position set to the start of where you
     *                   want to read and the limit set to the end of where you want to read
     * @return An index buffer
     */
    public BufferInfo getOrCreateIndexBuffer(ManagedBuffer<ByteBuffer> indexBytes) {
        ByteBuffer bytes = indexBytes.buffer();
        int previousPosition = bytes.position();
        int hash = BUFFER_HASH_FUNCTION.hashBytes(bytes).asInt();
        bytes.position(previousPosition);
        indexHashToInvocationsFrameMap.computeIfAbsent(hash, i -> new AtomicInteger()).incrementAndGet();
        BufferInfo buffer = indexHashToBufferMap.get(hash);
        if (buffer == null) {
            buffer = createIndexBuffer(indexBytes);
            indexHashToBufferMap.put(hash, buffer);
        } else {
            indexBytes.free(common.device, memory);
        }
        return buffer;
    }

    /**
     * Gets or creates a vertex buffer
     *
     * @param vertexBytes The bytes representing the vertices.
     *                    This buffer must have the position set to the start of where you
     *                    want to read and the limit set to the end of where you want to read
     * @return A vertex buffer
     */
    public BufferInfo getOrCreateVertexBuffer(ManagedBuffer<ByteBuffer> vertexBytes) {
        ByteBuffer bytes = vertexBytes.buffer();
        int previousPosition = bytes.position();
        int hash = BUFFER_HASH_FUNCTION.hashBytes(bytes).asInt();
        bytes.position(previousPosition);
        vertexHashToInvocationsFrameMap.computeIfAbsent(hash, i -> new AtomicInteger()).incrementAndGet();
        BufferInfo buffer = vertexHashToBufferMap.get(hash);
        if (buffer == null) {
            buffer = createVertexBuffer(vertexBytes);
            vertexHashToBufferMap.put(hash, buffer);
        } else {
            vertexBytes.free(common.device, memory);
        }
        return buffer;
    }

    /**
     * Creates a index buffer
     *
     * @param indexBytes The bytes representing the indices.
     *                   This buffer must have the position set to the start of where you
     *                   want to read and the limit set to the end of where you want to read
     * @return An index buffer
     */
    public BufferInfo createIndexBuffer(ManagedBuffer<ByteBuffer> indexBytes) {
        ByteBuffer src = indexBytes.buffer();
        int size = src.limit() - src.position();

        try (MemoryStack stack = stackPush()) {
            LongBuffer pBuffer = stack.mallocLong(1);

            BufferInfo stagingBuffer = memory.createStagingBuf(size, pBuffer, data -> {
                ByteBuffer dst = data.getByteBuffer(0, size);
                // TODO OPT: do optional batching again
                dst.put(0, src, src.position(), size);
            });
            indexBytes.free(common.device, memory);

            BufferInfo indexBuffer = memory.createBuffer(
                    size,
                    VK_BUFFER_USAGE_TRANSFER_DST_BIT | VK_BUFFER_USAGE_INDEX_BUFFER_BIT,
                    VMA_MEMORY_USAGE_GPU_ONLY,
                    pBuffer
            );

            long pIndexBuffer = pBuffer.get(0);
            memory.copyBuffer(stagingBuffer.buffer(),
                    pIndexBuffer,
                    size,
                    renderer,
                    common.device);
            stagingBuffer.free(common.device, memory);

            return indexBuffer;
        }
    }

    /**
     * Creates a vertex buffer.
     *
     * @param vertexBytes The bytes representing the vertices.
     *                    This buffer must have the position set to the start of where you
     *                    want to read and the limit set to the end of where you want to read
     * @return A vertex buffer
     */
    public BufferInfo createVertexBuffer(ManagedBuffer<ByteBuffer> vertexBytes) {
        ByteBuffer src = vertexBytes.buffer();
        int size = src.limit() - src.position();

        try (MemoryStack stack = stackPush()) {
            LongBuffer pBuffer = stack.mallocLong(1);

            BufferInfo stagingBuffer = memory.createStagingBuf(size, pBuffer, data -> {
                ByteBuffer dst = data.getByteBuffer(0, size);
                // TODO OPT: do optional batching again
                dst.put(0, src, src.position(), size);
            });
            vertexBytes.free(common.device, memory);

            BufferInfo vertexBuffer = memory.createBuffer(
                    size,
                    VK_BUFFER_USAGE_TRANSFER_DST_BIT | VK_BUFFER_USAGE_VERTEX_BUFFER_BIT,
                    VMA_MEMORY_USAGE_GPU_ONLY,
                    pBuffer
            );

            long pVertexBuffer = pBuffer.get(0);
            memory.copyBuffer(stagingBuffer.buffer(),
                    pVertexBuffer,
                    size,
                    renderer,
                    common.device);
            stagingBuffer.free(common.device, memory);

            return vertexBuffer;
        }
    }
}
