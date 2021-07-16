/**
 * This file is for accessing vulkan indirectly. it manages structs so engine code can look better.
 */
@file:JvmName("VkKt")

package me.hydos.rosella.render

import me.hydos.rosella.device.QueueFamilyIndices
import me.hydos.rosella.device.VulkanDevice
import me.hydos.rosella.memory.Memory
import me.hydos.rosella.render.renderer.Renderer
import me.hydos.rosella.render.swapchain.DepthBuffer
import me.hydos.rosella.render.swapchain.RenderPass
import me.hydos.rosella.render.swapchain.Swapchain
import me.hydos.rosella.render.texture.ImageRegion
import me.hydos.rosella.render.texture.Texture
import me.hydos.rosella.render.texture.TextureImage
import me.hydos.rosella.render.texture.UploadableImage
import me.hydos.rosella.util.Color
import me.hydos.rosella.util.VkConc
import org.lwjgl.PointerBuffer
import org.lwjgl.system.MemoryStack
import org.lwjgl.vulkan.*
import org.lwjgl.vulkan.VK10.*
import java.nio.LongBuffer

fun allocateCmdBuffers(
    device: VulkanDevice,
    commandPool: Long,
    commandBuffersCount: Int,
    level: Int = VK_COMMAND_BUFFER_LEVEL_PRIMARY
): PointerBuffer {
    return VkConc.allocateCommandBuffers(device, commandPool, commandBuffersCount, level)
}

fun createBeginInfo(stack: MemoryStack): VkCommandBufferBeginInfo {
    return VkCommandBufferBeginInfo.callocStack(stack)
        .sType(VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO)
}

fun createRenderPassInfo(stack: MemoryStack, renderPass: RenderPass): VkRenderPassBeginInfo {
    return VkRenderPassBeginInfo.callocStack(stack)
        .sType(VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO)
        .renderPass(renderPass.renderPass)
}

fun createRenderArea(stack: MemoryStack, x: Int = 0, y: Int = 0, swapchain: Swapchain): VkRect2D {
    return VkRect2D.callocStack(stack)
        .offset(VkOffset2D.callocStack(stack).set(x, y))
        .extent(swapchain.swapChainExtent)
}

fun createImageView(image: Long, format: Int, aspectFlags: Int, device: VulkanDevice): Long {
    return VkConc.createImageView(device, image, format, aspectFlags)
}

fun createImgViews(swapchain: Swapchain, device: VulkanDevice) {
    VkConc.createImageViews(device, swapchain)
}

fun createCmdPool(device: VulkanDevice, renderer: Renderer, surface: Long) {
    VkConc.createCommandPool(device, renderer, surface)
}

fun createClearValues(
    r: Float = 0f,
    g: Float = 0f,
    b: Float = 0f,
    depth: Float = 1.0f,
    stencil: Int = 0
): VkClearValue.Buffer {
    return VkConc.createClearValues(Color(r, g, b, 0F), depth, stencil)
}

fun findQueueFamilies(device: VkDevice, surface: Long): QueueFamilyIndices {
    return findQueueFamilies(device.physicalDevice, surface)
}

fun findQueueFamilies(device: VulkanDevice, surface: Long): QueueFamilyIndices {
    return findQueueFamilies(device.physicalDevice, surface)
}

fun findQueueFamilies(device: VkPhysicalDevice, surface: Long): QueueFamilyIndices {
    return VkConc.findQueueFamilies(device, surface)
}

fun createTextureImageView(device: VulkanDevice, imgFormat: Int, textureImage: Long): Long {
    return createImageView(
        textureImage,
        imgFormat,
        VK_IMAGE_ASPECT_COLOR_BIT,
        device
    )
}

fun createImage(
    width: Int, height: Int, format: Int, tiling: Int, usage: Int, memProperties: Int,
    pTextureImage: LongBuffer, pTextureImageMemory: LongBuffer, device: VulkanDevice
) {
    val info = VkConc.createImage(device, width, height, format, tiling, usage, memProperties)
    pTextureImage.put(0, info.buffer)
    pTextureImageMemory.put(0, info.allocation)
}

fun transitionImageLayout(
    renderer: Renderer,
    device: VulkanDevice,
    depthBuffer: DepthBuffer,
    image: Long,
    format: Int,
    oldLayout: Int,
    newLayout: Int
) {
    return VkConc.transitionImageLayout(device, renderer, depthBuffer, image, format, oldLayout, newLayout)
}

fun createTextureImage(
    renderer: Renderer,
    device: VulkanDevice,
    width: Int,
    height: Int,
    imgFormat: Int,
    textureImage: TextureImage
) {
    VkConc.createTextureImage(device, renderer, width, height, imgFormat, textureImage)
}

fun copyToTexture(
    renderer: Renderer,
    device: VulkanDevice,
    memory: Memory,
    image: UploadableImage,
    srcRegion: ImageRegion,
    dstRegion: ImageRegion,
    texture: Texture
) {
    VkConc.copyToTexture(device, renderer, memory, image, srcRegion, dstRegion, texture)
}

fun copyBufferToImage(
    renderer: Renderer,
    device: VulkanDevice,
    buffer: Long,
    image: Long,
    srcImageWidth: Int,
    srcImageHeight: Int,
    srcXOffset: Int,
    srcYOffset: Int,
    srcPixelSize: Int,
    dstRegionWidth: Int,
    dstRegionHeight: Int,
    dstXOffset: Int,
    dstYOffset: Int
) {
    return VkConc.copyBufferToImage(
        device,
        renderer,
        buffer,
        image,
        srcImageWidth,
        srcImageHeight,
        srcXOffset,
        srcYOffset,
        srcPixelSize,
        dstRegionWidth,
        dstRegionHeight,
        dstXOffset,
        dstYOffset
    )
}
