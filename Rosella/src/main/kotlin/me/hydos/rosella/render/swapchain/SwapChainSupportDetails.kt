package me.hydos.rosella.render.swapchain

import org.lwjgl.vulkan.VkSurfaceCapabilitiesKHR
import org.lwjgl.vulkan.VkSurfaceFormatKHR
import java.nio.IntBuffer

class SwapChainSupportDetails {
	lateinit var capabilities: VkSurfaceCapabilitiesKHR
	lateinit var formats: VkSurfaceFormatKHR.Buffer
	lateinit var presentModes: IntBuffer
}