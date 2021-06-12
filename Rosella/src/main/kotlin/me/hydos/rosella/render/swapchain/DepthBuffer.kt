package me.hydos.rosella.render.swapchain

import me.hydos.rosella.render.createImage
import me.hydos.rosella.render.createImageView
import me.hydos.rosella.render.transitionImageLayout
import me.hydos.rosella.render.device.Device
import me.hydos.rosella.render.renderer.Renderer
import org.lwjgl.system.MemoryStack
import org.lwjgl.vulkan.VK10.*
import org.lwjgl.vulkan.VkFormatProperties
import java.nio.IntBuffer

/**
 * Since vulkan gives us so much control, we must make our own depth buffer instead of relying on the driver to create one for us.
 */
class DepthBuffer {

	var depthImage: Long = 0
	var depthImageMemory: Long = 0
	var depthImageView: Long = 0

	fun createDepthResources(device: Device, swapChain: SwapChain, renderer: Renderer) {
		MemoryStack.stackPush().use { stack ->
			val depthFormat: Int = findDepthFormat(device)
			val pDepthImage = stack.mallocLong(1)
			val pDepthImageMemory = stack.mallocLong(1)
			createImage(
				swapChain.swapChainExtent.width(),
				swapChain.swapChainExtent.height(),
				depthFormat,
				VK_IMAGE_TILING_OPTIMAL,
				VK_IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT,
				VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT,
				pDepthImage,
				pDepthImageMemory,
				device
			)
			depthImage = pDepthImage[0]
			depthImageMemory = pDepthImageMemory[0]
			depthImageView = createImageView(depthImage, depthFormat, VK_IMAGE_ASPECT_DEPTH_BIT, device)

			// Explicitly transitioning the depth image
			transitionImageLayout(
				depthImage,
				depthFormat,
				VK_IMAGE_LAYOUT_UNDEFINED,
				VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
				renderer.depthBuffer,
				device,
				renderer
			)
		}
	}

	fun findDepthFormat(device: Device): Int {
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

	private fun findSupportedFormat(formatCandidates: IntBuffer, tiling: Int, features: Int, device: Device): Int {
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

	fun free(device: Device) {
		vkDestroyImageView(device.device, depthImageView, null)
		vkDestroyImage(device.device, depthImage, null)
		vkFreeMemory(device.device, depthImageMemory, null)
	}
}