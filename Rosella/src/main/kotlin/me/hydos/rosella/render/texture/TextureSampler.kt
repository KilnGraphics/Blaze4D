package me.hydos.rosella.render.texture

import me.hydos.rosella.device.VulkanDevice
import org.lwjgl.system.MemoryStack
import org.lwjgl.vulkan.VK10
import org.lwjgl.vulkan.VkSamplerCreateInfo

/**
 * The creation info for creating a Texture Sampler
 */
class TextureSampler(private val createInfo: SamplerCreateInfo, device: VulkanDevice) {
	var pointer = 0L

	init {
		MemoryStack.stackPush().use { stack ->
			val samplerInfo = VkSamplerCreateInfo.callocStack(stack)
				.sType(VK10.VK_STRUCTURE_TYPE_SAMPLER_CREATE_INFO)
				.magFilter(createInfo.filter.vkType)
				.minFilter(createInfo.filter.vkType)
				.addressModeU(createInfo.wrap.vkType)
				.addressModeV(createInfo.wrap.vkType)
				.addressModeW(createInfo.wrap.vkType)
				.anisotropyEnable(true)
				.maxAnisotropy(16.0f)
				.borderColor(VK10.VK_BORDER_COLOR_INT_OPAQUE_BLACK)
				.unnormalizedCoordinates(false)
				.compareEnable(false)
				.compareOp(VK10.VK_COMPARE_OP_ALWAYS)
			if (createInfo.filter.vkType == VK10.VK_FILTER_LINEAR) {
				samplerInfo.mipmapMode(VK10.VK_SAMPLER_MIPMAP_MODE_LINEAR)
			} else {
				samplerInfo.mipmapMode(VK10.VK_SAMPLER_MIPMAP_MODE_NEAREST)
			}
			val pTextureSampler = stack.mallocLong(1)
			if (VK10.vkCreateSampler(device.rawDevice, samplerInfo, null, pTextureSampler) != VK10.VK_SUCCESS) {
				throw RuntimeException("Failed to create texture sampler")
			}
			pointer = pTextureSampler[0]
		}
	}
}

data class SamplerCreateInfo(val filter: TextureFilter, val wrap: WrapMode)

enum class TextureFilter(val vkType: Int) {
	NEAREST(VK10.VK_FILTER_NEAREST),
	LINEAR(VK10.VK_FILTER_LINEAR)
}

enum class WrapMode(val vkType: Int) {
	REPEAT(VK10.VK_SAMPLER_ADDRESS_MODE_REPEAT),
	MIRRORED_REPEAT(VK10.VK_SAMPLER_ADDRESS_MODE_MIRRORED_REPEAT),
	CLAMP_TO_EDGE(VK10.VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE),
	CLAMP_TO_BORDER(VK10.VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_BORDER)
}
