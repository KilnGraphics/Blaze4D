package me.hydos.rosella.render.descriptorsets

import me.hydos.rosella.device.VulkanDevice
import org.lwjgl.vulkan.VK10

class DescriptorSet(var descriptorPool: Long = 0L) {
	var descriptorSets = ArrayList<Long>()

	fun free(device: VulkanDevice) {
		if (descriptorPool != 0L) {

			val listIterator = descriptorSets.listIterator()
			for (descriptorSet in listIterator) {
				if (descriptorSet != 0L) {
					VK10.vkFreeDescriptorSets(device.rawDevice, descriptorPool, descriptorSet)
				}
				listIterator.remove()
			}
		}
	}

	fun add(descriptorSet: Long) {
		descriptorSets.add(descriptorSet)
	}
}