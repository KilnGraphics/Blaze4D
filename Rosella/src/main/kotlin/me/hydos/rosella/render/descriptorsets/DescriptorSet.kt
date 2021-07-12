package me.hydos.rosella.render.descriptorsets

import me.hydos.rosella.device.VulkanDevice
import org.lwjgl.vulkan.VK10

class DescriptorSet(var descriptorPool: Long? = null) {
    var descriptorSets = ArrayList<Long>()

    fun free(device: VulkanDevice) {
        descriptorPool?.also {
            for (descriptorSet in descriptorSets) {
                if (descriptorSet != 0L) {
                    VK10.vkFreeDescriptorSets(device.rawDevice, it, descriptorSet)
                }
            }

            descriptorSets.clear()
        }

        descriptorPool = null
    }

    fun add(descriptorSet: Long) {
        descriptorSets.add(descriptorSet)
    }
}
