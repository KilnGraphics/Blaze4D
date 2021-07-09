package me.hydos.rosella.memory;

import me.hydos.rosella.Rosella;
import me.hydos.rosella.device.VulkanDevice;
import me.hydos.rosella.render.renderer.Renderer;
import me.hydos.rosella.render.vertex.BufferVertexConsumer;
import me.hydos.rosella.render.vertex.VertexConsumer;
import me.hydos.rosella.vkobjects.VkCommon;
import org.lwjgl.PointerBuffer;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.system.Pointer;
import org.lwjgl.util.vma.Vma;
import org.lwjgl.util.vma.VmaAllocationCreateInfo;
import org.lwjgl.util.vma.VmaAllocatorCreateInfo;
import org.lwjgl.util.vma.VmaVulkanFunctions;
import org.lwjgl.vulkan.*;

import java.nio.ByteBuffer;
import java.nio.LongBuffer;
import java.util.*;
import java.util.concurrent.ConcurrentLinkedQueue;
import java.util.function.Consumer;

import static me.hydos.rosella.render.util.VkUtilsKt.ok;
import static org.lwjgl.system.MemoryStack.stackPush;

/**
 * Used for managing CPU and GPU memory.
 * This class will try to handle most vma stuff for the user so they dont have to touch much memory related stuff
 */
public class Memory {
    private static final int THREAD_COUNT = 3;

    private final long allocator;
    private final VkCommon common;
    private final List<Long> mappedMemory = new ArrayList<>();
    private final List<Thread> workers = new ArrayList<>(THREAD_COUNT);
    private final Queue<Consumer<Long>> deallocationQueue = new ConcurrentLinkedQueue<>();
    private boolean running = true;

    public Memory(VkCommon common) {
        this.common = common;
        allocator = createAllocator(common);

        for (int i = 0; i < THREAD_COUNT; i++) {
            Thread thread = new Thread(() -> {
                long threadAllocator = createAllocator(common);

                while (running) {
                    Consumer<Long> consumer = deallocationQueue.poll();

                    if (consumer != null) {
                        consumer.accept(threadAllocator);
                    }
                }

                Vma.vmaDestroyAllocator(threadAllocator);
            }, "Deallocator Thread " + i);
            thread.start();
            workers.add(thread);
        }
    }

    /**
     * Converts a {@link List} into a {@link PointerBuffer}
     *
     * @param list  the list to put into a {@link PointerBuffer}
     * @param stack the current {@link MemoryStack}
     * @return a valid {@link PointerBuffer}
     */
    public static PointerBuffer asPtrBuffer(List<String> list, MemoryStack stack) {
        PointerBuffer pBuffer = stack.mallocPointer(list.size());
        for (String object : list) {
            pBuffer.put(Objects.requireNonNull(stack.UTF8Safe(object)));
        }
        return pBuffer.rewind();
    }

    /**
     * Converts a {@link Set} into a {@link PointerBuffer}
     *
     * @param set   the list to put into a {@link PointerBuffer}
     * @param stack the current {@link MemoryStack}
     * @return a valid {@link PointerBuffer}
     */
    public static PointerBuffer asPtrBuffer(Set<String> set, MemoryStack stack) {
        PointerBuffer buffer = stack.mallocPointer(set.size());
        for (String object : set) {
            buffer.put(stack.UTF8(object));
        }

        return buffer.rewind();
    }

    private long createAllocator(VkCommon common) {
        try (MemoryStack stack = stackPush()) {
            VmaVulkanFunctions vulkanFunctions = VmaVulkanFunctions.callocStack(stack)
                    .set(common.vkInstance.rawInstance, common.device.rawDevice);

            VmaAllocatorCreateInfo createInfo = VmaAllocatorCreateInfo.callocStack(stack)
                    .physicalDevice(common.device.physicalDevice)
                    .device(common.device.rawDevice)
                    .pVulkanFunctions(vulkanFunctions)
                    .instance(common.vkInstance.rawInstance)
                    .vulkanApiVersion(Rosella.VULKAN_VERSION);

            PointerBuffer pAllocator = stack.mallocPointer(1);
            Vma.vmaCreateAllocator(createInfo, pAllocator);

            Rosella.LOGGER.info("New allocator created. 0x%x", pAllocator.get(0));

            return pAllocator.get(0);
        }
    }

    /**
     * Maps an allocation with an Pointer Buffer
     */
    public void map(long allocation, boolean unmapOnClose, PointerBuffer data) {
        if (unmapOnClose) {
            mappedMemory.add(allocation);
        }

        Vma.vmaMapMemory(allocator, allocation, data);
    }

    /**
     * Unmaps allocated memory. this should usually be called on close
     */
    public void unmap(long allocation) {
        deallocationQueue.add(allocator -> {
            mappedMemory.remove(allocation);
            Vma.vmaUnmapMemory(allocator, allocation);
        });
    }

    /**
     * Used for creating the buffer written to before copied to the GPU
     */
    public BufferInfo createStagingBuf(int size, LongBuffer pBuffer, MemoryStack stack, Consumer<PointerBuffer> callback) {
        BufferInfo stagingBuffer = createBuffer(
                size,
                VK10.VK_BUFFER_USAGE_TRANSFER_SRC_BIT,
                Vma.VMA_MEMORY_USAGE_CPU_ONLY,
                pBuffer
        );
        PointerBuffer data = stack.mallocPointer(1);
        map(stagingBuffer.allocation(), true, data);
        callback.accept(data);
        return stagingBuffer;
    }


    /**
     * Used to create a Vulkan Memory Allocator Buffer.
     */
    public BufferInfo createBuffer(int size, int usage, int vmaUsage, LongBuffer pBuffer) {
        long allocation;
        try (MemoryStack stack = stackPush()) {
            if (size == 0) {
                throw new RuntimeException("Failed To Create VMA Buffer Reason: Buffer Size is 0");
            }

            VkBufferCreateInfo vulkanBufferInfo = VkBufferCreateInfo.callocStack(stack)
                    .sType(VK10.VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO)
                    .size(size)
                    .usage(usage)
                    .sharingMode(VK10.VK_SHARING_MODE_EXCLUSIVE);

            VmaAllocationCreateInfo vmaBufferInfo = VmaAllocationCreateInfo.callocStack(stack)
                    .usage(vmaUsage);

            PointerBuffer pAllocation = stack.mallocPointer(1);
            int result = Vma.vmaCreateBuffer(allocator, vulkanBufferInfo, vmaBufferInfo, pBuffer, pAllocation, null);
            if (result != 0) {
                throw new RuntimeException("Failed To Create VMA Buffer. Error Code " + result);
            }
            allocation = pAllocation.get(0);
        }
        return new BufferInfo(pBuffer.get(0), allocation);
    }

    /**
     * Copies a buffer from one place to another. usually used to copy a staging buffer into GPU mem
     */
    public void copyBuffer(long srcBuffer, long dstBuffer, int size, Renderer renderer, VulkanDevice device) {
        try (MemoryStack stack = stackPush()) {
            PointerBuffer pCommandBuffer = stack.mallocPointer(1);
            VkCommandBuffer commandBuffer = renderer.beginCmdBuffer(stack, pCommandBuffer, device);

            VkBufferCopy.Buffer copyRegion = VkBufferCopy.callocStack(1, stack);
            copyRegion.size(size);
            VK10.vkCmdCopyBuffer(commandBuffer, srcBuffer, dstBuffer, copyRegion);

            ok(VK10.vkEndCommandBuffer(commandBuffer));
            VkSubmitInfo submitInfo = VkSubmitInfo.callocStack(stack)
                    .sType(VK10.VK_STRUCTURE_TYPE_SUBMIT_INFO)
                    .pCommandBuffers(pCommandBuffer);
            ok(VK10.vkQueueSubmit(renderer.getQueues().graphicsQueue, submitInfo, VK10.VK_NULL_HANDLE));
            ok(VK10.vkQueueWaitIdle(renderer.getQueues().graphicsQueue));
            VK10.vkFreeCommandBuffers(device.rawDevice, renderer.getCommandPool(), pCommandBuffer);
        }
    }

    /**
     * Creates an index buffer from an list of indices
     */
    public BufferInfo createIndexBuffer(Rosella engine, List<Integer> indices) {
        try (MemoryStack stack = stackPush()) {
            int size = (Integer.BYTES * indices.size());
            LongBuffer pBuffer = stack.mallocLong(1);
            BufferInfo stagingBuffer = engine.common.memory.createStagingBuf(size, pBuffer, stack, data -> memcpy(data.getByteBuffer(0, size), indices));
            BufferInfo indexBufferInfo = createBuffer(
                    size,
                    VK10.VK_BUFFER_USAGE_TRANSFER_DST_BIT | VK10.VK_BUFFER_USAGE_INDEX_BUFFER_BIT,
                    Vma.VMA_MEMORY_USAGE_CPU_TO_GPU,
                    pBuffer
            );
            long indexBuffer = pBuffer.get(0);
            copyBuffer(stagingBuffer.buffer(), indexBuffer, size, engine.renderer, engine.common.device);
            stagingBuffer.free(common.device, this);
            return indexBufferInfo;
        }
    }

    /**
     * Forces a buffer to be freed
     */
    public void freeBuffer(BufferInfo buffer) {
        deallocationQueue.add(allocator -> Vma.vmaDestroyBuffer(allocator, buffer.buffer(), buffer.allocation()));
    }

    /**
     * Free's all created buffers and mapped memory
     */
    public void free() {
        for (long memory : mappedMemory) {
            unmap(memory);
        }

        running = false;

        for (Thread worker : workers) {
            try {
                worker.join();
            } catch (InterruptedException exception) {
                throw new RuntimeException(exception);
            }
        }

        for (Consumer<Long> consumer : deallocationQueue) {
            consumer.accept(allocator);
        }

        Vma.vmaDestroyAllocator(allocator);
    }

    /**
     * Copies indices into the specified buffer
     */
    public static void memcpy(ByteBuffer buffer, List<Integer> indices) {
        for (int index : indices) {
            buffer.putInt(index);
        }
    }

    /**
     * Copies an ByteBuffer into another ByteBuffer
     */
    public static void memcpy(ByteBuffer dst, ByteBuffer src, long size) {
        src.limit((int) size);
        dst.put(src);
        src.limit(src.capacity()).rewind();
    }

    public static PointerBuffer asPointerBuffer(List<? extends Pointer> pointers) {
        PointerBuffer buffer = MemoryStack.stackGet().mallocPointer(pointers.size());

        for (Pointer pointer : pointers) {
            buffer.put(pointer);
        }

        return buffer.rewind();
    }
}

