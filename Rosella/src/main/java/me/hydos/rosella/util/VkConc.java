package me.hydos.rosella.util;

import me.hydos.rosella.device.QueueFamilyIndices;
import me.hydos.rosella.device.VulkanDevice;
import me.hydos.rosella.memory.BufferInfo;
import me.hydos.rosella.memory.ImageInfo;
import me.hydos.rosella.memory.Memory;
import me.hydos.rosella.render.renderer.Renderer;
import me.hydos.rosella.render.swapchain.DepthBuffer;
import me.hydos.rosella.render.swapchain.Swapchain;
import me.hydos.rosella.render.texture.ImageRegion;
import me.hydos.rosella.render.texture.Texture;
import me.hydos.rosella.render.texture.TextureImage;
import me.hydos.rosella.render.texture.UploadableImage;
import org.lwjgl.PointerBuffer;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.util.vma.Vma;
import org.lwjgl.util.vma.VmaAllocationCreateInfo;
import org.lwjgl.vulkan.*;

import java.nio.ByteBuffer;
import java.nio.IntBuffer;
import java.nio.LongBuffer;
import java.util.ArrayList;

import static me.hydos.rosella.util.VulkanUtils.ok;
import static org.lwjgl.vulkan.VK10.*;

public class VkConc {

    public static PointerBuffer allocateCommandBuffers(VulkanDevice device, long pool, int count, int level) {
        try (MemoryStack stack = MemoryStack.stackPush()) {
            VkCommandBufferAllocateInfo allocInfo = VkCommandBufferAllocateInfo.callocStack()
                    .sType(VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO)
                    .commandPool(pool)
                    .level(level)
                    .commandBufferCount(count);
            PointerBuffer pointers = stack.callocPointer(count);
            ok(vkAllocateCommandBuffers(device.rawDevice, allocInfo, pointers));
            return pointers;
        }
    }

    public static Long createImageView(VulkanDevice device, long image, int format, int aspectFlags) {
        try (MemoryStack stack = MemoryStack.stackPush()) {
            VkImageViewCreateInfo createInfo = VkImageViewCreateInfo.callocStack(stack)
                    .sType(VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO)
                    .image(image)
                    .viewType(VK_IMAGE_VIEW_TYPE_2D)
                    .format(format)
                    .subresourceRange(subresourceRange -> subresourceRange.set(aspectFlags, 0, 1, 0, 1));
            LongBuffer buffer = stack.mallocLong(1);
            ok(vkCreateImageView(device.rawDevice, createInfo, null, buffer));
            return buffer.get(0);
        }
    }

    public static void createImageViews(VulkanDevice device, Swapchain swapchain) {
        ArrayList<Long> views = new ArrayList<>(swapchain.getSwapChainImages().size());
        swapchain.setSwapChainImageViews(views);

        for (long image : swapchain.getSwapChainImages()) {
            views.add(createImageView(device, image, swapchain.getSwapChainImageFormat(), VK_IMAGE_ASPECT_COLOR_BIT));
        }
    }

    public static void createCommandPool(VulkanDevice device, Renderer renderer, long surface) {
        try (MemoryStack stack = MemoryStack.stackPush()) {
            QueueFamilyIndices indices = findQueueFamilies(device.rawDevice.getPhysicalDevice(), surface);
            VkCommandPoolCreateInfo createInfo = VkCommandPoolCreateInfo.mallocStack(stack)
                    .sType(VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO)
                    .queueFamilyIndex(indices.graphicsFamily)
                    .pNext(VK_NULL_HANDLE);
            LongBuffer pCommandPool = stack.mallocLong(1);
            ok(vkCreateCommandPool(device.rawDevice, createInfo, null, pCommandPool));
            renderer.commandPool = pCommandPool.get(0);
        }
    }

    public static VkClearValue.Buffer createClearValues(Color color, float depth, int stencil) {
        VkClearValue.Buffer values = VkClearValue.callocStack(2);
        values.get(0).color().float32(MemoryStack.stackGet().floats(color.rAsFloat(), color.gAsFloat(), color.bAsFloat()));
        values.get(1).depthStencil().set(depth, stencil);
        return values;
    }

    public static QueueFamilyIndices findQueueFamilies(VkPhysicalDevice device, long surface) {
        try (MemoryStack stack = MemoryStack.stackPush()) {
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

    public static ImageInfo createImage(Memory memory, int width, int height, int format, int tiling, int usage, int memoryProperties, int vmaUsage) {
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

    public static void transitionImageLayout(VulkanDevice device, Renderer renderer, DepthBuffer depthBuffer, long image, int format, int oldLayout, int newLayout) {
        try (MemoryStack stack = MemoryStack.stackPush()) {
            VkImageMemoryBarrier.Buffer barrier = VkImageMemoryBarrier.mallocStack(1, stack)
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

            VkCommandBuffer commandBuffer = beginSingleTimeCommands(device, renderer);
            vkCmdPipelineBarrier(
                    commandBuffer,
                    sourceStage, destinationStage,
                    0,
                    null,
                    null,
                    barrier
            );
            endSingleTimeCommands(device, renderer, commandBuffer);
        }
    }

    public static void createTextureImage(VulkanDevice device, Memory memory, Renderer renderer, int width, int height, int format, TextureImage textureImage) {
        ImageInfo image = createImage(memory, width, height, format, VK_IMAGE_TILING_OPTIMAL, VK_IMAGE_USAGE_TRANSFER_DST_BIT | VK_IMAGE_USAGE_SAMPLED_BIT, VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT, Vma.VMA_MEMORY_USAGE_GPU_ONLY);
        textureImage.setTextureImage(image.buffer());
        textureImage.setTextureImageMemory(image.allocation());

        transitionImageLayout(device, renderer, renderer.depthBuffer, textureImage.getTextureImage(), format, VK_IMAGE_LAYOUT_UNDEFINED, VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL);
    }

    public static void copyToTexture(VulkanDevice device, Renderer renderer, Memory memory, UploadableImage image, ImageRegion sourceRegion, ImageRegion destRegion, Texture texture) {
        try (MemoryStack stack = MemoryStack.stackPush()) {
            BufferInfo stagingBuf = memory.createStagingBuf(image.getSize(), stack.mallocLong(1), stack, data -> {
                ByteBuffer pixels = image.getPixels();
                ByteBuffer newData = data.getByteBuffer(0, pixels.limit());
                newData.put(0, pixels, 0, pixels.limit());
            });

            copyBufferToImage(
                    device,
                    renderer,
                    stagingBuf.buffer(),
                    texture.getTextureImage().getTextureImage(),
                    image.getWidth(),
                    image.getHeight(),
                    sourceRegion.xOffset(),
                    sourceRegion.yOffset(),
                    image.getFormat().getPixelSize(),
                    destRegion.width(),
                    destRegion.height(),
                    destRegion.xOffset(),
                    destRegion.yOffset()
            );

            stagingBuf.free(device, memory);
        }
    }

    public static void copyBufferToImage(VulkanDevice device, Renderer renderer, long buffer, long image, int sourceImageWidth, int sourceImageHeight, int sourceXOffset, int sourceYOffset, int sourcePixelSize, int destRegionWidth, int destRegionHeight, int destXOffset, int destYOffset) {
        // TODO OPT: have image be linear tiling until it is prepared for the first time, then make it optimal
        try (MemoryStack stack = MemoryStack.stackPush()) {
            VkBufferImageCopy.Buffer region = VkBufferImageCopy.callocStack(1, stack)
                    .bufferOffset((((long) sourceYOffset * sourceImageWidth) + sourceXOffset) * sourcePixelSize)
                    .bufferRowLength(sourceImageWidth)
                    .bufferImageHeight(sourceImageHeight)
                    .imageOffset(imageOffset -> imageOffset.set(destXOffset, destYOffset, 0))
                    .imageSubresource(imageSubresource -> imageSubresource.set(VK_IMAGE_ASPECT_COLOR_BIT, 0, 0, 1))
                    .imageExtent(imageExtent -> imageExtent.set(destRegionWidth, destRegionHeight, 1));

            VkCommandBuffer commandBuffer = beginSingleTimeCommands(device, renderer);
            vkCmdCopyBufferToImage(commandBuffer, buffer, image, VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL, region);
            endSingleTimeCommands(device, renderer, commandBuffer);
        }
    }

    private static int findMemoryType(VulkanDevice device, int filter, int properties) {
        try (MemoryStack stack = MemoryStack.stackPush()) {
            VkPhysicalDeviceMemoryProperties memoryProperties = VkPhysicalDeviceMemoryProperties.mallocStack(stack);
            vkGetPhysicalDeviceMemoryProperties(device.physicalDevice, memoryProperties);

            for (int i = 0; i < memoryProperties.memoryTypeCount(); i++) {
                if ((filter & 1 << i) != 0 && (memoryProperties.memoryTypes(i).propertyFlags() & properties) == properties) {
                    return i;
                }
            }
        }

        throw new RuntimeException("Failed to find suitable memory type");
    }

    private static VkCommandBuffer beginSingleTimeCommands(VulkanDevice device, Renderer renderer) {
        try (MemoryStack stack = MemoryStack.stackPush()) {
            return renderer.beginCmdBuffer(stack, stack.mallocPointer(1), device);
        }
    }

    private static void endSingleTimeCommands(VulkanDevice device, Renderer renderer, VkCommandBuffer buffer) {
        try (MemoryStack stack = MemoryStack.stackPush()) {
            ok(vkEndCommandBuffer(buffer));
            VkSubmitInfo.Buffer submitInfo = VkSubmitInfo.callocStack(1, stack)
                    .sType(VK_STRUCTURE_TYPE_SUBMIT_INFO)
                    .pCommandBuffers(stack.pointers(buffer));
            renderer.queues.graphicsQueue.vkQueueSubmit(submitInfo, VK_NULL_HANDLE);
            renderer.queues.graphicsQueue.vkQueueWaitIdle();
            vkFreeCommandBuffers(device.rawDevice, renderer.commandPool, buffer);
        }
    }
}
