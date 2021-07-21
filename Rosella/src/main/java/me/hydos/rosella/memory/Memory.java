package me.hydos.rosella.memory;

import it.unimi.dsi.fastutil.longs.LongOpenHashSet;
import it.unimi.dsi.fastutil.longs.LongSet;
import it.unimi.dsi.fastutil.longs.LongSets;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.device.VulkanDevice;
import me.hydos.rosella.render.material.PipelineInfo;
import me.hydos.rosella.render.renderer.Renderer;
import me.hydos.rosella.render.texture.TextureImage;
import me.hydos.rosella.util.VkUtils;
import me.hydos.rosella.vkobjects.VkCommon;
import org.lwjgl.PointerBuffer;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.system.MemoryUtil;
import org.lwjgl.system.Pointer;
import org.lwjgl.util.vma.Vma;
import org.lwjgl.util.vma.VmaAllocationCreateInfo;
import org.lwjgl.util.vma.VmaAllocatorCreateInfo;
import org.lwjgl.util.vma.VmaVulkanFunctions;
import org.lwjgl.vulkan.*;

import java.nio.Buffer;
import java.nio.ByteBuffer;
import java.nio.LongBuffer;
import java.util.Collection;
import java.util.List;
import java.util.Objects;
import java.util.Set;
import java.util.concurrent.LinkedBlockingQueue;
import java.util.concurrent.ThreadPoolExecutor;
import java.util.concurrent.TimeUnit;
import java.util.function.Consumer;

import static me.hydos.rosella.util.VkUtils.ok;
import static org.lwjgl.system.MemoryStack.stackPush;

/**
 * Used for managing CPU and GPU memory.
 * This class will try to handle most vma stuff for the user so they dont have to touch much memory related stuff
 */
public abstract class Memory {
    private static final int THREAD_COUNT = 3;

    private final VkCommon common;
    private final LongSet mappedMemory = LongSets.synchronize(new LongOpenHashSet());

    private final long allocator;
    private final ThreadPoolExecutor deallocatorThreadPool;
    private int threadNo;

    private boolean running = true;

    public Memory(VkCommon common) {
        this.common = common;

        this.allocator = createAllocator(common);
        this.deallocatorThreadPool = new ThreadPoolExecutor(
                THREAD_COUNT,
                THREAD_COUNT,
                0L,
                TimeUnit.MILLISECONDS,
                new LinkedBlockingQueue<>(),
                r -> new Thread(r, "Deallocator Thread " + threadNo++),
                (r, executor) -> {/* noop */});
    }

    /**
     * Converts a {@link List} into a {@link PointerBuffer}
     *
     * @param collection  the collection to put into a {@link PointerBuffer}
     * @return a valid {@link PointerBuffer}
     */
    public static PointerBuffer asPtrBuffer(Collection<String> collection) {
        MemoryStack stack = MemoryStack.stackGet();
        PointerBuffer pBuffer = stack.mallocPointer(collection.size());
        for (String object : collection) {
            pBuffer.put(Objects.requireNonNull(stack.UTF8Safe(object)));
        }
        return pBuffer.rewind();
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

            return pAllocator.get(0);
        }
    }

    private void destroyAllocator(long allocator) {
        Vma.vmaDestroyAllocator(allocator);
    }

    /**
     * Maps an allocation with a Pointer Buffer
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
        deallocatorThreadPool.execute(() -> {
            mappedMemory.remove(allocation);
            Vma.vmaUnmapMemory(allocator, allocation);
        });
    }

    /**
     * Allocates an image buffer
     *
     * @param pImageCreateInfo Information related to the image which will be contained
     * @param vmaUsage The memory type provided to VMA
     * @return The bundle of the image and the allocation addresses
     */
    public TextureImage createImageBuffer(VkImageCreateInfo pImageCreateInfo, int memoryProperties, int vmaUsage) {
        try (MemoryStack stack = MemoryStack.stackPush()) {

            LongBuffer pTextureImage = stack.mallocLong(1);
            ok(VK10.vkCreateImage(common.device.rawDevice, pImageCreateInfo, null, pTextureImage), "Failed to allocate image memory");
            long textureImage = pTextureImage.get(0);

            VkMemoryRequirements requirements = VkMemoryRequirements.mallocStack(stack);
            VK10.vkGetImageMemoryRequirements(common.device.rawDevice, textureImage, requirements);
            VkMemoryAllocateInfo allocateInfo = VkMemoryAllocateInfo.callocStack(stack)
                    .sType(VK10.VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO)
                    .allocationSize(requirements.size())
                    .memoryTypeIndex(VkUtils.findMemoryType(common.device, requirements.memoryTypeBits(), memoryProperties));

            LongBuffer pTextureImageMemory = stack.mallocLong(1);
            ok(VK10.vkAllocateMemory(common.device.rawDevice, allocateInfo, null, pTextureImageMemory));
            long textureImageMemory = pTextureImageMemory.get(0);

//            LongBuffer pImage = stack.mallocLong(1);
//            PointerBuffer pAllocation = stack.mallocPointer(1);
//            // TODO OPT: try to make allocation create info more customizable
//            VmaAllocationCreateInfo pAllocationCreateInfo = VmaAllocationCreateInfo.mallocStack(stack)
//                    //.preferredFlags(memoryProperties)
//                    .usage(vmaUsage);
//            ok(Vma.vmaCreateImage(allocator, pImageCreateInfo, pAllocationCreateInfo, pImage, pAllocation, null), "Failed to allocate image memory");
//            long image = pImage.get(0);
//            long allocation = pAllocation.get(0);
//            ok(Vma.vmaBindImageMemory(allocator, textureImageMemory, textureImage), "Failed to bind image to memory");
            ok(VK10.vkBindImageMemory(common.device.rawDevice, textureImage, textureImageMemory, 0), "Failed to bind image to memory");
            return new TextureImage(textureImage, textureImageMemory, 0);
        }
    }

    /**
     * Used for creating the buffer written to before copied to the GPU
     */
    public BufferInfo createStagingBuf(int size, LongBuffer pBuffer, Consumer<PointerBuffer> callback) {
        try (MemoryStack stack = MemoryStack.stackPush()) {
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
            ok(Vma.vmaCreateBuffer(allocator, vulkanBufferInfo, vmaBufferInfo, pBuffer, pAllocation, null));
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
            VkCommandBuffer commandBuffer = renderer.beginCmdBuffer(pCommandBuffer, device);

            VkBufferCopy.Buffer copyRegion = VkBufferCopy.callocStack(1, stack);
            copyRegion.size(size);
            VK10.vkCmdCopyBuffer(commandBuffer, srcBuffer, dstBuffer, copyRegion);

            ok(VK10.vkEndCommandBuffer(commandBuffer));
            VkSubmitInfo submitInfo = VkSubmitInfo.callocStack(stack)
                    .sType(VK10.VK_STRUCTURE_TYPE_SUBMIT_INFO)
                    .pCommandBuffers(pCommandBuffer);
            ok(renderer.queues.graphicsQueue.vkQueueSubmit(submitInfo, VK10.VK_NULL_HANDLE));
            ok(renderer.queues.graphicsQueue.vkQueueWaitIdle());
            VK10.vkFreeCommandBuffers(device.rawDevice, renderer.commandPool, pCommandBuffer);
        }
    }

    /**
     * Queues a buffer to be freed
     */
    public void freeBuffer(BufferInfo buffer) {
        deallocatorThreadPool.execute(() -> Vma.vmaDestroyBuffer(allocator, buffer.buffer(), buffer.allocation()));
    }

    /**
     * Queues an image to be freed
     */
    public void freeImage(TextureImage image) {
        deallocatorThreadPool.execute(() -> {
            VK10.vkDestroyImage(common.device.rawDevice, image.pointer(), null);
            VK10.vkFreeMemory(common.device.rawDevice, image.getTextureImageMemory(), null);
//            Vma.vmaDestroyImage(allocator, image.pointer(), image.getTextureImageMemory());
            if (image.getView() != VK10.VK_NULL_HANDLE) {
                VK10.vkDestroyImageView(common.device.rawDevice, image.getView(), null);
            }
        });
    }

    public void freePipeline(PipelineInfo pipeline) {
//        deallocatorThreadPool.execute(() -> {
            VK10.vkDestroyPipeline(common.device.rawDevice, pipeline.graphicsPipeline(), null);
            VK10.vkDestroyPipelineLayout(common.device.rawDevice, pipeline.pipelineLayout(), null);
//        });
    }

    /**
     * Frees a LongArrayList of descriptor sets
     */
    public void freeDescriptorSets(long descriptorPool, ManagedBuffer<LongBuffer> descriptorSets) {
//        deallocatorThreadPool.execute(() -> {
            // FIXME synchronize
            VK10.vkFreeDescriptorSets(common.device.rawDevice, descriptorPool, descriptorSets.buffer().flip());
            descriptorSets.free(common.device, this);
//        });
    }

    /**
     * Frees a ManagedBuffer in a deallocator thread
     */
    public void freeDirectBufferAsync(Buffer buffer) {
        deallocatorThreadPool.execute(() -> MemoryUtil.memFree(buffer));
    }

    /**
     * Frees all created buffers and mapped memory
     */
    public void free() {
        for (long memory : mappedMemory) {
            unmap(memory);
        }

        running = false;

        deallocatorThreadPool.shutdown();
        try {
            // the time gets converted to nanos anyway, so avoid long overflow
            if (!deallocatorThreadPool.awaitTermination(Long.MAX_VALUE, TimeUnit.NANOSECONDS)) {
                Rosella.LOGGER.debug("Memory thread pool took too long to shut down");
            }
        } catch (InterruptedException e) {
            Rosella.LOGGER.debug("Error shutting down memory thread pool");
        }

        destroyAllocator(allocator);
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

