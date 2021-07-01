package me.hydos.rosella.render.shader.ubo

import me.hydos.rosella.render.`object`.RenderObject
import me.hydos.rosella.render.descriptorsets.DescriptorSet
import me.hydos.rosella.render.device.Device
import me.hydos.rosella.render.shader.ShaderProgram
import me.hydos.rosella.render.swapchain.Swapchain
import me.hydos.rosella.render.util.alignas
import me.hydos.rosella.render.util.alignof
import me.hydos.rosella.render.util.memory.BufferInfo
import me.hydos.rosella.render.util.memory.Memory
import me.hydos.rosella.render.util.sizeof
import org.joml.Matrix4f
import org.lwjgl.system.MemoryStack
import org.lwjgl.util.vma.Vma
import org.lwjgl.vulkan.VK10

open class RenderObjectUbo(val device: Device, val memory: Memory, private val renderObject: RenderObject, shaderProgram: ShaderProgram) : Ubo() {

	var uboFrames: MutableList<BufferInfo> = ArrayList()
	var descSets: DescriptorSet = DescriptorSet(shaderProgram.raw.descriptorPool)

	override fun create(swapchain: Swapchain) {
		MemoryStack.stackPush().use { stack ->
			uboFrames = ArrayList(swapchain.swapChainImages.size)
			for (i in swapchain.swapChainImages.indices) {
				val pBuffer = stack.mallocLong(1)
				uboFrames.add(
					memory.createBuffer(
						getSize(),
						VK10.VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT,
						Vma.VMA_MEMORY_USAGE_CPU_ONLY,
						pBuffer
					)
				)
			}
		}
	}

	override fun update(currentImg: Int, swapchain: Swapchain) {
		if (uboFrames.size == 0) {
			create(swapchain) //TODO: CONCERN. why did i write this
		}

		MemoryStack.stackPush().use {
			val data = it.mallocPointer(1)
			memory.map(uboFrames[currentImg].allocation, false, data)
			val buffer = data.getByteBuffer(0, getSize())
			val mat4Size = 16 * java.lang.Float.BYTES
			renderObject.modelMatrix[0, buffer]
			renderObject.viewMatrix.get(alignas(mat4Size, alignof(renderObject.viewMatrix)), buffer)
			renderObject.projectionMatrix.get(alignas(mat4Size * 2, alignof(renderObject.projectionMatrix)), buffer)
			memory.unmap(uboFrames[currentImg].allocation)
		}
	}

	override fun free() {
		for (uboImg in uboFrames) {
			memory.freeBuffer(uboImg)
		}
	}

	override fun getSize(): Int {
		return 3 * sizeof(Matrix4f::class)
	}

	override fun getUniformBuffers(): List<BufferInfo> {
		return uboFrames
	}

	override fun getDescriptors(): DescriptorSet {
		return descSets
	}

	override fun setDescriptors(descriptorSets: DescriptorSet) {
		this.descSets = descriptorSets
	}
}