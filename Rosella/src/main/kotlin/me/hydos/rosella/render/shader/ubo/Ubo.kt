package me.hydos.rosella.render.shader.ubo

import me.hydos.rosella.device.VulkanDevice
import me.hydos.rosella.memory.BufferInfo
import me.hydos.rosella.memory.Memory
import me.hydos.rosella.memory.MemoryCloseable
import me.hydos.rosella.render.descriptorsets.DescriptorSets
import me.hydos.rosella.render.swapchain.Swapchain

/**
 * A Uniform Buffer Object (ubo) is an object used to do things such as sending transformation matrices to the shader, sending lighting values to the shader, etc
 */
abstract class Ubo : MemoryCloseable {

    /**
     * Called when the uniform buffers should be created
     */
    abstract fun create(swapchain: Swapchain)

    /**
     * Called before each frame to update the ubo
     */
    abstract fun update(currentImg: Int, swapchain: Swapchain)

    /**
     * Called when the program is closing and free's memory
     */
    abstract fun free()

    /**
     * Gets the size of the ubo
     */
    abstract fun getSize(): Int

    /**
     * Gets an list of pointers to the ubo frames
     */
    abstract fun getUniformBuffers(): List<BufferInfo>

    /**
     * Gets the descriptor sets used with this ubo
     */
    abstract fun getDescriptors(): DescriptorSets

    /**
     * Called when the program is closing and free's memory
     */
    override fun free(device: VulkanDevice?, memory: Memory?) {
        free()
    }

    abstract fun setDescriptors(descriptorSets: DescriptorSets)
}
