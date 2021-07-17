package me.hydos.rosella.util;

import it.unimi.dsi.fastutil.longs.LongArrayList;
import me.hydos.rosella.device.QueueFamilyIndices;
import me.hydos.rosella.device.VulkanDevice;
import me.hydos.rosella.memory.BufferInfo;
import me.hydos.rosella.memory.Memory;
import me.hydos.rosella.render.renderer.Renderer;
import me.hydos.rosella.render.swapchain.DepthBuffer;
import me.hydos.rosella.render.swapchain.RenderPass;
import me.hydos.rosella.render.swapchain.Swapchain;
import me.hydos.rosella.render.texture.ImageRegion;
import me.hydos.rosella.render.texture.Texture;
import me.hydos.rosella.render.texture.TextureImage;
import me.hydos.rosella.render.texture.UploadableImage;
import org.lwjgl.PointerBuffer;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.util.vma.Vma;
import org.lwjgl.vulkan.*;

import java.nio.ByteBuffer;
import java.nio.IntBuffer;
import java.nio.LongBuffer;
import java.util.Map;

import static java.util.Map.entry;
import static java.util.Map.ofEntries;
import static org.lwjgl.system.MemoryStack.*;
import static org.lwjgl.vulkan.VK10.*;

public class VkUtils {
    private static final Map<Integer, String> ERROR_NAMES = ofEntries(
            entry(VK10.VK_NOT_READY, "VK_NOT_READY"),
            entry(VK10.VK_TIMEOUT, "VK_TIMEOUT"),
            entry(VK10.VK_EVENT_SET, "VK_EVENT_SET"),
            entry(VK10.VK_EVENT_RESET, "VK_EVENT_RESET"),
            entry(VK10.VK_INCOMPLETE, "VK_INCOMPLETE"),
            entry(VK10.VK_ERROR_OUT_OF_HOST_MEMORY, "VK_ERROR_OUT_OF_HOST_MEMORY"),
            entry(VK11.VK_ERROR_OUT_OF_POOL_MEMORY, "VK_ERROR_OUT_OF_POOL_MEMORY"),
            entry(VK10.VK_ERROR_OUT_OF_DEVICE_MEMORY, "VK_ERROR_OUT_OF_DEVICE_MEMORY"),
            entry(VK10.VK_ERROR_INITIALIZATION_FAILED, "VK_ERROR_INITIALIZATION_FAILED"),
            entry(VK10.VK_ERROR_DEVICE_LOST, "VK_ERROR_DEVICE_LOST"),
            entry(VK10.VK_ERROR_MEMORY_MAP_FAILED, "VK_ERROR_MEMORY_MAP_FAILED"),
            entry(VK10.VK_ERROR_LAYER_NOT_PRESENT, "VK_ERROR_LAYER_NOT_PRESENT"),
            entry(VK10.VK_ERROR_EXTENSION_NOT_PRESENT, "VK_ERROR_EXTENSION_NOT_PRESENT"),
            entry(VK10.VK_ERROR_FEATURE_NOT_PRESENT, "VK_ERROR_FEATURE_NOT_PRESENT"),
            entry(VK10.VK_ERROR_INCOMPATIBLE_DRIVER, "VK_ERROR_INCOMPATIBLE_DRIVER"),
            entry(VK10.VK_ERROR_TOO_MANY_OBJECTS, "VK_ERROR_TOO_MANY_OBJECTS"),
            entry(VK10.VK_ERROR_FORMAT_NOT_SUPPORTED, "VK_ERROR_FORMAT_NOT_SUPPORTED"),
            entry(VK10.VK_ERROR_FRAGMENTED_POOL, "VK_ERROR_FRAGMENTED_POOL"),
            entry(VK10.VK_ERROR_UNKNOWN, "VK_ERROR_UNKNOWN"),
            entry(KHRSurface.VK_ERROR_NATIVE_WINDOW_IN_USE_KHR, "VK_ERROR_NATIVE_WINDOW_IN_USE_KHR"),
            entry(EXTDebugReport.VK_ERROR_VALIDATION_FAILED_EXT, "VK_ERROR_VALIDATION_FAILED_EXT")
    );

    public static void ok(int vkResult) {
        if (vkResult != VK10.VK_SUCCESS) {
            throw new RuntimeException(ERROR_NAMES.getOrDefault(vkResult, Integer.toString(vkResult)));
        }
    }

    public static void ok(int vkResult, String message) {
        if (vkResult != VK10.VK_SUCCESS) {
            throw new RuntimeException(message + ", caused by " + ERROR_NAMES.getOrDefault(vkResult, Integer.toString(vkResult)));
        }
    }

    public static PointerBuffer allocateCommandBuffers(VulkanDevice device, long commandPool, int commandBuffersCount, int level) {
        PointerBuffer pCommandBuffers = stackGet().callocPointer(commandBuffersCount);
        try (MemoryStack stack = stackPush()) {
            VkCommandBufferAllocateInfo allocInfo = VkCommandBufferAllocateInfo.callocStack(stack)
                    .sType(VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO)
                    .commandPool(commandPool)
                    .level(level)
                    .commandBufferCount(commandBuffersCount);

            ok(vkAllocateCommandBuffers(device.rawDevice, allocInfo, pCommandBuffers));
        }
        return pCommandBuffers;
    }

    public static PointerBuffer allocateCommandBuffers(VulkanDevice device, long commandPool, int commandBuffersCount) {
        return allocateCommandBuffers(device, commandPool, commandBuffersCount, VK_COMMAND_BUFFER_LEVEL_PRIMARY);
    }

    public static VkCommandBufferBeginInfo createBeginInfo() {
        return VkCommandBufferBeginInfo.callocStack()
                .sType(VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO);
    }

    public static VkRenderPassBeginInfo createRenderPassInfo(RenderPass renderPass) {
        return VkRenderPassBeginInfo.callocStack()
                .sType(VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO)
                .renderPass(renderPass.getRenderPass());
    }

    public static VkRect2D createRenderArea(int x, int y, Swapchain swapchain) {
        return VkRect2D.callocStack()
                .offset(VkOffset2D.callocStack().set(x, y))
                .extent(swapchain.getSwapChainExtent());
    }

    public static VkRect2D createRenderArea(Swapchain swapchain) {
        return createRenderArea(0, 0, swapchain);
    }

    public static long createImageView(VulkanDevice device, long image, int format, int aspectFlags) {
        try (MemoryStack stack = stackPush()) {
            VkImageViewCreateInfo viewInfo = VkImageViewCreateInfo.callocStack(stack)
                    .sType(VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO)
                    .image(image)
                    .viewType(VK_IMAGE_VIEW_TYPE_2D)
                    .format(format)
                    .subresourceRange(subresourceRange ->
                            subresourceRange
                                    .aspectMask(aspectFlags)
                                    .baseMipLevel(0)
                                    .levelCount(1)
                                    .baseArrayLayer(0)
                                    .layerCount(1)
                    );

            LongBuffer pImageView = stack.mallocLong(1);
            ok(vkCreateImageView(device.rawDevice, viewInfo, null, pImageView), "Failed to create texture image view");
            return pImageView.get(0);
        }
    }

    public static void createSwapchainImageViews(Swapchain swapchain, VulkanDevice device) {
        swapchain.setSwapChainImageViews(new LongArrayList(swapchain.getSwapChainImages().size()));

        for (long swapChainImage : swapchain.getSwapChainImages()) {
            swapchain.getSwapChainImageViews().add(
                    createImageView(
                            device,
                            swapChainImage,
                            swapchain.getSwapChainImageFormat(),
                            VK_IMAGE_ASPECT_COLOR_BIT
                    )
            );
        }
    }

    public static long createTextureImageView(VulkanDevice device, int imgFormat, long textureImage) {
        return createImageView(device, textureImage, imgFormat, VK_IMAGE_ASPECT_COLOR_BIT);
    }

    public static QueueFamilyIndices findQueueFamilies(VkDevice device, long surface) {
        return findQueueFamilies(device.getPhysicalDevice(), surface);
    }

    public static QueueFamilyIndices findQueueFamilies(VulkanDevice device, long surface) {
        return findQueueFamilies(device.physicalDevice, surface);
    }

    public static QueueFamilyIndices findQueueFamilies(VkPhysicalDevice device, long surface) {
        try (MemoryStack stack = stackPush()) {
            QueueFamilyIndices indices = new QueueFamilyIndices();

            IntBuffer queueFamilyCount = stack.ints(0);
            vkGetPhysicalDeviceQueueFamilyProperties(device, queueFamilyCount, null);

            VkQueueFamilyProperties.Buffer queueFamilies = VkQueueFamilyProperties.mallocStack(queueFamilyCount.get(0), stack);
            vkGetPhysicalDeviceQueueFamilyProperties(device, queueFamilyCount, queueFamilies);

            IntBuffer presentSupport = stack.ints(VK_FALSE);

            for (int i = 0; i < queueFamilies.capacity() || !indices.isComplete(); i++) {
                if ((queueFamilies.get(i).queueFlags() & VK_QUEUE_GRAPHICS_BIT) != 0) {
                    indices.graphicsFamily = i;
                }
                KHRSurface.vkGetPhysicalDeviceSurfaceSupportKHR(device, i, surface, presentSupport);
                if (presentSupport.get(0) == VK_TRUE) {
                    indices.presentFamily = i;
                }
            }

            return indices;
        }
    }

    public static void createCommandPool(VulkanDevice device, Renderer renderer, long surface) {
        try (MemoryStack stack = stackPush()) {
            QueueFamilyIndices queueFamilyIndices = findQueueFamilies(device, surface);
            VkCommandPoolCreateInfo poolInfo = VkCommandPoolCreateInfo.callocStack(stack)
                    .sType(VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO)
                    .queueFamilyIndex(queueFamilyIndices.graphicsFamily);

            LongBuffer pCommandPool = stack.mallocLong(1);
            ok(vkCreateCommandPool(device.rawDevice, poolInfo, null, pCommandPool));

            renderer.commandPool = pCommandPool.get(0);
        }
    }

    public static VkClearValue.Buffer createClearValues(float r, float g, float b, float depth, int stencil) {
        VkClearValue.Buffer clearValues = VkClearValue.callocStack(2);
        clearValues.get(0).color().float32(stackGet().floats(r, g, b, 1.0f));
        clearValues.get(1).depthStencil().set(depth, stencil);
        return clearValues;
    }

    public static VkCommandBuffer beginSingleTimeCommands(Renderer renderer, VulkanDevice device) {
        MemoryStack stack = stackGet();
        PointerBuffer pCommandBuffer = stack.mallocPointer(1);
        return renderer.beginCmdBuffer(stack, pCommandBuffer, device);
    }

    public static void endSingleTimeCommands(VkCommandBuffer commandBuffer, VulkanDevice device, Renderer renderer) {
        try (MemoryStack stack = stackPush()) {
            vkEndCommandBuffer(commandBuffer);
            VkSubmitInfo.Buffer submitInfo = VkSubmitInfo.callocStack(1, stack)
                    .sType(VK_STRUCTURE_TYPE_SUBMIT_INFO)
                    .pCommandBuffers(stack.pointers(commandBuffer));
            renderer.queues.graphicsQueue.vkQueueSubmit(submitInfo, VK_NULL_HANDLE);
            renderer.queues.graphicsQueue.vkQueueWaitIdle();
            vkFreeCommandBuffers(device.rawDevice, renderer.commandPool, commandBuffer);
        }
    }

    public static int findMemoryType(int typeFilter, int properties, VulkanDevice device) {
        VkPhysicalDeviceMemoryProperties memProperties = VkPhysicalDeviceMemoryProperties.mallocStack();
        vkGetPhysicalDeviceMemoryProperties(device.physicalDevice, memProperties);
        for (int i = 0; i < memProperties.memoryTypeCount(); i++) {
            if ((typeFilter & (1 << i)) != 0 && (memProperties.memoryTypes(i).propertyFlags() & properties) == properties) {
                return i;
            }
        }
        throw new IllegalStateException("Failed to find suitable memory type");
    }

    public static TextureImage createImage(Memory memory, int width, int height, int format, int tiling, int usage, int memoryProperties, int vmaUsage) {
        try (MemoryStack stack = MemoryStack.stackPush()) {
            VkImageCreateInfo imageInfo = VkImageCreateInfo.callocStack(stack)
                    .sType(VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO)
                    .imageType(VK_IMAGE_TYPE_2D)
                    .extent(extent -> extent.set(width, height, 1))
                    .mipLevels(1)
                    .arrayLayers(1)
                    .format(format)
                    .tiling(tiling)
                    .initialLayout(VK_IMAGE_LAYOUT_UNDEFINED)
                    .usage(usage)
                    .samples(VK_SAMPLE_COUNT_1_BIT)
                    .sharingMode(VK_SHARING_MODE_EXCLUSIVE);
            // TODO OPT: figure out how vma pools work
            return memory.createImageBuffer(imageInfo, memoryProperties, vmaUsage);
        }
    }

    public static TextureImage createTextureImage(Renderer renderer, Memory memory, VulkanDevice device, int width, int height, int imgFormat) {
        TextureImage image = createImage(
                memory,
                width,
                height,
                imgFormat,
                VK_IMAGE_TILING_OPTIMAL,
                VK_IMAGE_USAGE_TRANSFER_DST_BIT | VK_IMAGE_USAGE_SAMPLED_BIT,
                VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT,
                Vma.VMA_MEMORY_USAGE_GPU_ONLY // FIXME
        );

        transitionImageLayout(
                renderer,
                device,
                renderer.depthBuffer,
                image.pointer(),
                imgFormat,
                VK_IMAGE_LAYOUT_UNDEFINED,
                VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL
        );

        return image;
    }

    public static void transitionImageLayout(Renderer renderer, VulkanDevice device, DepthBuffer depthBuffer, long image, int format, int oldLayout, int newLayout) {
        try (MemoryStack stack = stackPush()) {
            VkImageMemoryBarrier.Buffer barrier = VkImageMemoryBarrier.callocStack(1, stack)
                    .sType(VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER)
                    .oldLayout(oldLayout)
                    .newLayout(newLayout)
                    .srcQueueFamilyIndex(VK_QUEUE_FAMILY_IGNORED)
                    .dstQueueFamilyIndex(VK_QUEUE_FAMILY_IGNORED)
                    .image(image)
                    .subresourceRange(subresourceRange -> {
                        subresourceRange
                                .baseMipLevel(0)
                                .levelCount(1)
                                .baseArrayLayer(0)
                                .layerCount(1);
                        if (newLayout == VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL) {
                            subresourceRange.aspectMask(VK_IMAGE_ASPECT_DEPTH_BIT | (depthBuffer.hasStencilComponent(format) ? VK_IMAGE_ASPECT_STENCIL_BIT : 0));
                        } else {
                            subresourceRange.aspectMask(VK_IMAGE_ASPECT_COLOR_BIT);
                        }
                    });

            int sourceStage;
            int destinationStage;
            if (oldLayout == VK_IMAGE_LAYOUT_UNDEFINED && newLayout == VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL) {

                barrier.srcAccessMask(0)
                        .dstAccessMask(VK_ACCESS_TRANSFER_WRITE_BIT);

                sourceStage = VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT;
                destinationStage = VK_PIPELINE_STAGE_TRANSFER_BIT;

            } else if (oldLayout == VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL && newLayout == VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL) {

                barrier.srcAccessMask(VK_ACCESS_TRANSFER_WRITE_BIT)
                        .dstAccessMask(VK_ACCESS_SHADER_READ_BIT);

                sourceStage = VK_PIPELINE_STAGE_TRANSFER_BIT;
                destinationStage = VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT;

            } else if (oldLayout == VK_IMAGE_LAYOUT_UNDEFINED && newLayout == VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL) {

                barrier.srcAccessMask(0)
                        .dstAccessMask(VK_ACCESS_DEPTH_STENCIL_ATTACHMENT_READ_BIT | VK_ACCESS_DEPTH_STENCIL_ATTACHMENT_WRITE_BIT);

                sourceStage = VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT;
                destinationStage = VK_PIPELINE_STAGE_EARLY_FRAGMENT_TESTS_BIT;

            } else if (oldLayout == VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL && newLayout == VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL) {

                barrier.srcAccessMask(VK_ACCESS_SHADER_READ_BIT)
                        .dstAccessMask(VK_ACCESS_TRANSFER_WRITE_BIT);

                sourceStage = VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT;
                destinationStage = VK_PIPELINE_STAGE_TRANSFER_BIT;

            } else {
                throw new IllegalArgumentException("Unsupported layout transition");
            }
            VkCommandBuffer commandBuffer = beginSingleTimeCommands(renderer, device);
            vkCmdPipelineBarrier(commandBuffer, sourceStage, destinationStage, 0, null, null, barrier);
            VkUtils.endSingleTimeCommands(commandBuffer, device, renderer);
        }
    }

    public static void copyToTexture(Renderer renderer, VulkanDevice device, Memory memory, UploadableImage image, ImageRegion srcRegion, ImageRegion dstRegion, Texture texture) {
        try (MemoryStack stack = stackPush()) {
            LongBuffer pBuffer = stack.mallocLong(1);
            BufferInfo stagingBuf = memory.createStagingBuf(
                    image.getSize(),
                    pBuffer,
                    stack,
                    data -> {
                        ByteBuffer pixels = image.getPixels();
                        ByteBuffer newData = data.getByteBuffer(0, pixels.limit());
                        newData.put(0, pixels, 0, pixels.limit());
                    }
            );

            copyBufferToImage(
                    renderer,
                    device,
                    stagingBuf.buffer(),
                    texture.getTextureImage().pointer(),
                    image.getWidth(),
                    image.getHeight(),
                    srcRegion.xOffset(),
                    srcRegion.yOffset(),
                    image.getFormat().getPixelSize(),
                    dstRegion.width(),
                    dstRegion.height(),
                    dstRegion.xOffset(),
                    dstRegion.yOffset()
            );

            stagingBuf.free(device, memory);
        }
    }

    public static void copyBufferToImage(Renderer renderer, VulkanDevice device, long buffer, long image, int srcImageWidth, int srcImageHeight, int srcXOffset, int srcYOffset, int srcPixelSize, int dstRegionWidth, int dstRegionHeight, int dstXOffset, int dstYOffset) {
        // TODO: add support for mip levels
        // TODO OPT: use linear layout until first prepare, then keep it at optimal. copying to linear is faster but reading is slower.
        try (MemoryStack stack = stackPush()) {
            VkBufferImageCopy.Buffer region = VkBufferImageCopy.callocStack(1, stack)
                    .bufferOffset((((long) srcYOffset * srcImageWidth) + srcXOffset) * srcPixelSize)
                    .bufferRowLength(srcImageWidth)
                    .bufferImageHeight(srcImageHeight)
                    .imageOffset(VkOffset3D.mallocStack(stack).set(dstXOffset, dstYOffset, 0))
                    .imageSubresource(imageSubresource ->
                            imageSubresource
                                    .aspectMask(VK_IMAGE_ASPECT_COLOR_BIT)
                                    .mipLevel(0)
                                    .baseArrayLayer(0)
                                    .layerCount(1)
                    )
                    .imageExtent(VkExtent3D.mallocStack(stack).set(dstRegionWidth, dstRegionHeight, 1));

            VkCommandBuffer commandBuffer = beginSingleTimeCommands(renderer, device);
            vkCmdCopyBufferToImage(commandBuffer, buffer, image, VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL, region);
            VkUtils.endSingleTimeCommands(commandBuffer, device, renderer);
        }
    }
}
