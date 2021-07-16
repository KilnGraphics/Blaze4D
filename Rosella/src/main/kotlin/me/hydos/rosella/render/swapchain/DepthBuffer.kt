package me.hydos.rosella.render.swapchain

import me.hydos.rosella.device.VulkanDevice
import me.hydos.rosella.memory.Memory
import me.hydos.rosella.render.renderer.Renderer
import me.hydos.rosella.util.VkConc
import org.lwjgl.system.MemoryStack
import org.lwjgl.util.vma.Vma
import org.lwjgl.vulkan.VK10.*
import org.lwjgl.vulkan.VkFormatProperties
import java.nio.IntBuffer

/**
 * Since vulkan gives us so much control, we must make our own depth buffer instead of relying on the driver to create one for us.
 */
class DepthBuffer {

    private var depthImage: Long = 0
    private var depthImageMemory: Long = 0
    var depthImageView: Long = 0

    fun createDepthResources(device: VulkanDevice, memory: Memory, swapchain: Swapchain, renderer: Renderer) {
        val depthFormat: Int = findDepthFormat(device)
        val c = VkConc.createImage(
            memory,
            swapchain.swapChainExtent.width(),
            swapchain.swapChainExtent.height(),
            depthFormat,
            VK_IMAGE_TILING_OPTIMAL,
            VK_IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT,
            VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT,
            Vma.VMA_MEMORY_USAGE_GPU_ONLY
        )
        depthImage = c.buffer
        depthImageMemory = c.allocation
        depthImageView = VkConc.createImageView(device, depthImage, depthFormat, VK_IMAGE_ASPECT_DEPTH_BIT)

        // Explicitly transitioning the depth image
        VkConc.transitionImageLayout(
            device,
            renderer,
            renderer.depthBuffer,
            depthImage,
            depthFormat,
            VK_IMAGE_LAYOUT_UNDEFINED,
            VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL
        )
    }

    fun findDepthFormat(device: VulkanDevice): Int {
        return findSupportedFormat(
            MemoryStack.stackGet()
                .ints(VK_FORMAT_D32_SFLOAT, VK_FORMAT_D32_SFLOAT_S8_UINT, VK_FORMAT_D24_UNORM_S8_UINT),
            VK_IMAGE_TILING_OPTIMAL,
            VK_FORMAT_FEATURE_DEPTH_STENCIL_ATTACHMENT_BIT,
            device
        )
    }

    fun hasStencilComponent(format: Int): Boolean {
        return format == VK_FORMAT_D32_SFLOAT_S8_UINT || format == VK_FORMAT_D24_UNORM_S8_UINT
    }

    private fun findSupportedFormat(
        formatCandidates: IntBuffer,
        tiling: Int,
        features: Int,
        device: VulkanDevice
    ): Int {
        MemoryStack.stackPush().use { stack ->
            val props = VkFormatProperties.callocStack(stack)
            for (i in 0 until formatCandidates.capacity()) {
                val format = formatCandidates[i]
                vkGetPhysicalDeviceFormatProperties(device.physicalDevice, format, props)
                if (tiling == VK_IMAGE_TILING_LINEAR && props.linearTilingFeatures() and features == features) {
                    return format
                } else if (tiling == VK_IMAGE_TILING_OPTIMAL && props.optimalTilingFeatures() and features == features) {
                    return format
                }
            }
        }
        error("Failed to find supported format")
    }

    fun free(device: VulkanDevice) {
        vkDestroyImageView(device.rawDevice, depthImageView, null)
        vkDestroyImage(device.rawDevice, depthImage, null)
        vkFreeMemory(device.rawDevice, depthImageMemory, null)
    }
}
