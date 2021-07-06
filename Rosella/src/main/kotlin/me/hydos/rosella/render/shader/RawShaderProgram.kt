package me.hydos.rosella.render.shader

import me.hydos.rosella.device.VulkanDevice
import me.hydos.rosella.render.descriptorsets.DescriptorSet
import me.hydos.rosella.render.resource.Resource
import me.hydos.rosella.render.shader.ubo.Ubo
import me.hydos.rosella.render.swapchain.Swapchain
import me.hydos.rosella.render.texture.Texture
import me.hydos.rosella.render.util.memory.Memory
import me.hydos.rosella.render.util.ok
import me.hydos.rosella.scene.`object`.impl.SimpleObjectManager
import org.lwjgl.system.MemoryStack
import org.lwjgl.vulkan.*
import org.lwjgl.vulkan.VK10.*

open class RawShaderProgram(
	var vertexShader: Resource?,
	var fragmentShader: Resource?,
	val device: VulkanDevice,
	val memory: Memory,
	var maxObjCount: Int,
	vararg var poolObjects: PoolObjType
) {
	var descriptorPool: Long = 0
	var descriptorSetLayout: Long = 0

	fun updateUbos(currentImage: Int, swapchain: Swapchain, objectManager: SimpleObjectManager) {
		for (instances in objectManager.renderObjects.values) {
			for (instance in instances) {
				instance.ubo.update(
					currentImage,
					swapchain
				)
			}
		}
	}

	fun createPool(swapchain: Swapchain) {
		if(descriptorPool != 0L) {
			vkDestroyDescriptorPool(device.rawDevice, descriptorPool, null)
		}
		MemoryStack.stackPush().use { stack ->
			val poolSizes = VkDescriptorPoolSize.callocStack(poolObjects.size, stack)

			poolObjects.forEachIndexed { i, poolObj ->
				poolSizes[i]
					.type(poolObj.vkType)
					.descriptorCount(swapchain.swapChainImages.size * maxObjCount)
			}

			val poolInfo = VkDescriptorPoolCreateInfo.callocStack(stack)
				.sType(VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO)
				.pPoolSizes(poolSizes)
				.maxSets(swapchain.swapChainImages.size * maxObjCount)
				.flags(VK_DESCRIPTOR_POOL_CREATE_FREE_DESCRIPTOR_SET_BIT)

			val pDescriptorPool = stack.mallocLong(1)
			vkCreateDescriptorPool(
				device.rawDevice,
				poolInfo,
				null,
				pDescriptorPool
			).ok("Failed to create descriptor pool")

			descriptorPool = pDescriptorPool[0]
		}
	}

	fun createDescriptorSetLayout() {
		MemoryStack.stackPush().use {
			val bindings = VkDescriptorSetLayoutBinding.callocStack(poolObjects.size, it)

			poolObjects.forEachIndexed { i, poolObj ->
				bindings[i]
					.binding(i)
					.descriptorCount(1)
					.descriptorType(poolObj.vkType)
					.pImmutableSamplers(null)
					.stageFlags(poolObj.vkShader)
			}

			val layoutInfo = VkDescriptorSetLayoutCreateInfo.callocStack(it)
			layoutInfo.sType(VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO)
			layoutInfo.pBindings(bindings)
			val pDescriptorSetLayout = it.mallocLong(1)
			vkCreateDescriptorSetLayout(
				device.rawDevice,
				layoutInfo,
				null,
				pDescriptorSetLayout
			).ok("Failed to create descriptor set layout")
			descriptorSetLayout = pDescriptorSetLayout[0]
		}
	}

	fun createDescriptorSets(swapchain: Swapchain, logger: org.apache.logging.log4j.Logger, texture: Texture, ubo: Ubo) {
		if(descriptorPool == 0L) {
			logger.warn("Descriptor Pools are invalid! rebuilding... (THIS IS NOT FAST)")
			createPool(swapchain)
		}
		if(descriptorSetLayout == 0L) {
			logger.warn("Descriptor Set Layouts are invalid! rebuilding... (THIS IS NOT FAST)")
			createDescriptorSetLayout()
		}
		MemoryStack.stackPush().use { stack ->
			val layouts = stack.mallocLong(swapchain.swapChainImages.size)
			for (i in 0 until layouts.capacity()) {
				layouts.put(i, descriptorSetLayout)
			}
			val allocInfo = VkDescriptorSetAllocateInfo.callocStack(stack)
				.sType(VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO)
				.descriptorPool(descriptorPool)
				.pSetLayouts(layouts)
			val pDescriptorSets = stack.mallocLong(swapchain.swapChainImages.size)

			vkAllocateDescriptorSets(device.rawDevice, allocInfo, pDescriptorSets)
				.ok("Failed to allocate descriptor sets")

			val descriptorSets = DescriptorSet(descriptorPool)
			descriptorSets.descriptorSets = ArrayList(pDescriptorSets.capacity())

			val bufferInfo = VkDescriptorBufferInfo.callocStack(1, stack)
				.offset(0)
				.range(ubo.getSize().toLong())

			val imageInfo = VkDescriptorImageInfo.callocStack(1, stack)
				.imageLayout(VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL)
				.imageView(texture.textureImage.view)
				.sampler(texture.textureSampler!!)

			val descriptorWrites = VkWriteDescriptorSet.callocStack(poolObjects.size, stack)

			for (i in 0 until pDescriptorSets.capacity()) {
				val descriptorSet = pDescriptorSets[i]
				bufferInfo.buffer(ubo.getUniformBuffers()[i].buffer)
				poolObjects.forEachIndexed { index, poolObj ->
					val descriptorWrite = descriptorWrites[index]
						.sType(VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET)
						.dstBinding(index)
						.dstArrayElement(0)
						.descriptorType(poolObj.vkType)
						.descriptorCount(1)

					when (poolObj.vkType) {
						VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER -> {
							descriptorWrite.pBufferInfo(bufferInfo)
						}

						VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER -> {
							descriptorWrite.pImageInfo(imageInfo)
						}
					}
					descriptorWrite.dstSet(descriptorSet)
				}
				vkUpdateDescriptorSets(device.rawDevice, descriptorWrites, null)
				descriptorSets.descriptorPool = descriptorPool
				descriptorSets.add(descriptorSet)
			}

			ubo.setDescriptors(descriptorSets)
		}
	}

	fun free() {
		vkDestroyDescriptorSetLayout(device.rawDevice, descriptorSetLayout, null)
		vkDestroyDescriptorPool(device.rawDevice, descriptorPool, null)
	}

	enum class PoolObjType(val vkType: Int, val vkShader: Int) {
		UBO(VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER, VK_SHADER_STAGE_ALL),
		SAMPLER(VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER, VK_SHADER_STAGE_ALL)
	}
}