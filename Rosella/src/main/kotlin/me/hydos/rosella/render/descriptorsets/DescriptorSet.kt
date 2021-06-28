package me.hydos.rosella.render.descriptorsets

import me.hydos.rosella.render.device.Device
import org.lwjgl.vulkan.VK10

class DescriptorSet(var descriptorPool: Long = 0L) {
	var descriptorSets = ArrayList<Long>()

	fun free(device: Device) {
		for (descriptorSet in descriptorSets) {
			VK10.vkFreeDescriptorSets(device.device, descriptorPool, descriptorSet)
		}
	}

	fun add(descriptorSet: Long) {
		descriptorSets.add(descriptorSet)
	}
}