package me.hydos.rosella.render.swapchain

import me.hydos.rosella.device.QueueFamilyIndices
import me.hydos.rosella.display.Display
import me.hydos.rosella.util.VkConc
import me.hydos.rosella.util.VulkanUtils.ok
import org.lwjgl.system.MemoryStack
import org.lwjgl.vulkan.*
import org.lwjgl.vulkan.KHRSurface.*
import org.lwjgl.vulkan.KHRSwapchain.*
import org.lwjgl.vulkan.VK10.*
import java.nio.IntBuffer
import java.nio.LongBuffer

class Swapchain(
    display: Display,
    device: VkDevice,
    physicalDevice: VkPhysicalDevice,
    surface: Long
) {
    private var maxImages: IntBuffer
    var swapChain: Long = 0
    var swapChainImageViews: MutableList<Long> = ArrayList()
    var frameBuffers: MutableList<Long> = ArrayList()
    var swapChainImages: MutableList<Long> = ArrayList()
    var swapChainImageFormat = 0
    var swapChainExtent: VkExtent2D

    init {
        MemoryStack.stackPush().use {
            val swapchainSupport: SwapchainSupportDetails = querySwapchainSupport(physicalDevice, it, surface)

            val surfaceFormat: VkSurfaceFormatKHR = chooseSwapSurfaceFormat(swapchainSupport.formats)!!
            val presentMode: Int = chooseSwapPresentMode(swapchainSupport.presentModes, display.doVsync)
            val extent: VkExtent2D = chooseSwapExtent(swapchainSupport.capabilities, display)!!

            val imageCount: IntBuffer = it.ints(swapchainSupport.capabilities.minImageCount() + 1)
            this.maxImages = imageCount

            if (swapchainSupport.capabilities.maxImageCount() > 0 && imageCount[0] > swapchainSupport.capabilities.maxImageCount()) {
                imageCount.put(0, swapchainSupport.capabilities.maxImageCount())
            }

            val createInfo = VkSwapchainCreateInfoKHR.callocStack(it)
                .sType(VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR)
                .surface(surface)
                .minImageCount(imageCount[0])
                .imageFormat(surfaceFormat.format())
                .imageColorSpace(surfaceFormat.colorSpace())
                .imageExtent(extent)
                .imageArrayLayers(1)
                .imageUsage(VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT)

            val indices: QueueFamilyIndices = VkConc.findQueueFamilies(device.physicalDevice, surface)

            if (indices.graphicsFamily != indices.presentFamily) {
                createInfo.imageSharingMode(VK_SHARING_MODE_CONCURRENT)
                    .pQueueFamilyIndices(it.ints(indices.graphicsFamily, indices.presentFamily))
            } else {
                createInfo.imageSharingMode(VK_SHARING_MODE_EXCLUSIVE)
            }

            createInfo.preTransform(swapchainSupport.capabilities.currentTransform())
                .compositeAlpha(VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR)
                .presentMode(presentMode)
                .clipped(true)
                .oldSwapchain(VK_NULL_HANDLE)

            val pSwapChain: LongBuffer = it.longs(VK_NULL_HANDLE)
            ok(vkCreateSwapchainKHR(device, createInfo, null, pSwapChain))
            swapChain = pSwapChain[0]
            ok(vkGetSwapchainImagesKHR(device, swapChain, imageCount, null))
            val pSwapchainImages: LongBuffer = it.mallocLong(imageCount[0])
            ok(vkGetSwapchainImagesKHR(device, swapChain, imageCount, pSwapchainImages))

            swapChainImages = ArrayList(imageCount[0])

            for (i in 0 until pSwapchainImages.capacity()) {
                swapChainImages.add(pSwapchainImages[i])
            }

            swapChainImageFormat = surfaceFormat.format()
            swapChainExtent = VkExtent2D.create().set(extent)
        }
    }

    private fun chooseSwapSurfaceFormat(availableFormats: VkSurfaceFormatKHR.Buffer): VkSurfaceFormatKHR? {
        return availableFormats.stream()
            .filter { availableFormat: VkSurfaceFormatKHR -> availableFormat.format() == VK_FORMAT_B8G8R8_SRGB }
            .filter { availableFormat: VkSurfaceFormatKHR -> availableFormat.colorSpace() == VK_COLOR_SPACE_SRGB_NONLINEAR_KHR }
            .findAny()
            .orElse(availableFormats[0])
    }

    private fun chooseSwapPresentMode(availablePresentModes: IntBuffer, doVsync: Boolean): Int {
        val presentTable = if (doVsync) VSYNC_PREFERRED_PRESENT_TABLE else NO_VSYNC_PREFERRED_PRESENT_TABLE

        for (presentMode in presentTable) {
            for (i in 0 until availablePresentModes.capacity()) {
                if (availablePresentModes[i] == presentMode) {
                    return presentMode
                }
            }
        }
        return -1 // this should never hit
    }

    private fun chooseSwapExtent(capabilities: VkSurfaceCapabilitiesKHR, display: Display): VkExtent2D? {
        if (capabilities.currentExtent().width() != UINT32_MAX) {
            return capabilities.currentExtent()
        }

        val actualExtent = VkExtent2D.mallocStack().set(display.width, display.height)
        val minExtent = capabilities.minImageExtent()
        val maxExtent = capabilities.maxImageExtent()
        actualExtent.width(minExtent.width().coerceIn(maxExtent.width(), actualExtent.width()))
        actualExtent.height(minExtent.height().coerceIn(maxExtent.height(), actualExtent.height()))
        return actualExtent
    }

    fun free(device: VkDevice) {
        vkDestroySwapchainKHR(device, swapChain, null)
    }

    companion object {
        private const val UINT32_MAX = -0x1
        private val VSYNC_PREFERRED_PRESENT_TABLE: Array<Int> = arrayOf(
            VK_PRESENT_MODE_MAILBOX_KHR,
            VK_PRESENT_MODE_FIFO_KHR,
            VK_PRESENT_MODE_FIFO_RELAXED_KHR,
            VK_PRESENT_MODE_IMMEDIATE_KHR
        )
        private val NO_VSYNC_PREFERRED_PRESENT_TABLE: Array<Int> = arrayOf(
            VK_PRESENT_MODE_IMMEDIATE_KHR,
            VK_PRESENT_MODE_FIFO_RELAXED_KHR,
            VK_PRESENT_MODE_MAILBOX_KHR,
            VK_PRESENT_MODE_FIFO_KHR
        )

        fun querySwapchainSupport(
            device: VkPhysicalDevice,
            stack: MemoryStack,
            surface: Long
        ): SwapchainSupportDetails {
            val details = SwapchainSupportDetails()
            details.capabilities = VkSurfaceCapabilitiesKHR.mallocStack(stack)
            ok(vkGetPhysicalDeviceSurfaceCapabilitiesKHR(device, surface, details.capabilities))
            val count = stack.ints(0)
            ok(vkGetPhysicalDeviceSurfaceFormatsKHR(device, surface, count, null))
            if (count[0] != 0) {
                details.formats = VkSurfaceFormatKHR.mallocStack(count[0], stack)
                ok(vkGetPhysicalDeviceSurfaceFormatsKHR(device, surface, count, details.formats))
            }
            ok(vkGetPhysicalDeviceSurfacePresentModesKHR(device, surface, count, null))
            if (count[0] != 0) {
                details.presentModes = stack.mallocInt(count[0])
                ok(vkGetPhysicalDeviceSurfacePresentModesKHR(device, surface, count, details.presentModes))
            }
            return details
        }
    }
}
