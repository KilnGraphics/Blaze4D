package me.hydos.rosella.render.descriptorsets

import me.hydos.rosella.device.VulkanDevice
import org.lwjgl.vulkan.VK10
import java.util.concurrent.CompletableFuture

class DescriptorSet(var descriptorPool: Long? = null) {
	var descriptorSets = ArrayList<Long>()

	fun free(device: VulkanDevice) {
		descriptorPool?.also {
			val _descriptorSets = ArrayList(descriptorSets)
			CompletableFuture.runAsync {
				for (descriptorSet in _descriptorSets) {
					if (descriptorSet != 0L) {
						VK10.vkFreeDescriptorSets(device.rawDevice, it, descriptorSet)
					}
				}
			}.complete(null)
		}

		descriptorSets.clear()

		descriptorPool = null
	}

	fun clear() {
		descriptorSets.clear()
		descriptorPool = null
	}

	fun add(descriptorSet: Long) {
		descriptorSets.add(descriptorSet)
	}
}
