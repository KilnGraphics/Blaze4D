package me.hydos.rosella.render.shader

import it.unimi.dsi.fastutil.Hash.VERY_FAST_LOAD_FACTOR
import it.unimi.dsi.fastutil.objects.ReferenceOpenHashSet
import me.hydos.rosella.device.VulkanDevice
import me.hydos.rosella.memory.Memory
import me.hydos.rosella.render.descriptorsets.DescriptorSets
import me.hydos.rosella.render.renderer.Renderer
import me.hydos.rosella.render.resource.Resource
import me.hydos.rosella.render.shader.ubo.Ubo
import me.hydos.rosella.render.swapchain.Swapchain
import me.hydos.rosella.render.texture.Texture
import me.hydos.rosella.render.texture.TextureManager
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
    private vararg var poolObjects: PoolObjectInfo
) {
    var descriptorPool: Long = 0
    var descriptorSetLayout: Long = 0

    private val preparableTextures = ReferenceOpenHashSet<Texture?>(3, VERY_FAST_LOAD_FACTOR)

    fun updateUbos(currentImage: Int, swapchain: Swapchain, objectManager: SimpleObjectManager) {
        for (renderObject in objectManager.renderObjects) {
            renderObject.value().ubo.update(
                currentImage,
                swapchain
            )
        }
    }

    fun prepareTexturesForRender(
        renderer: Renderer,
        textureManager: TextureManager
    ) { // TODO: should we move this?
        preparableTextures.forEach {
            if (it != null) {
                textureManager.prepareTexture(renderer, it)
            }
        }
        preparableTextures.clear()
    }

    private fun createPool(swapchain: Swapchain) {
        if (descriptorPool != 0L) {
            vkDestroyDescriptorPool(device.rawDevice, descriptorPool, null)
        }
        MemoryStack.stackPush().use { stack ->
            val poolSizes = VkDescriptorPoolSize.callocStack(poolObjects.size, stack)

            poolObjects.forEachIndexed { i, poolObj ->
                poolSizes[i]
                    .type(poolObj.getVkType())
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
                    .binding(if (poolObj.getBindingLocation() == -1) i else poolObj.getBindingLocation())
                    .descriptorCount(1)
                    .descriptorType(poolObj.getVkType())
                    .pImmutableSamplers(null)
                    .stageFlags(poolObj.getShaderStage())
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

    fun createDescriptorSets(
        swapchain: Swapchain,
        logger: org.apache.logging.log4j.Logger,
        currentTextures: Array<Texture?>,
        ubo: Ubo
    ) {
        this.preparableTextures.addAll(currentTextures)

        if (descriptorPool == 0L) {
            createPool(swapchain)
        }
        if (descriptorSetLayout == 0L) {
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

            val descriptorSets = DescriptorSets(descriptorPool, pDescriptorSets.capacity())
            val bufferInfo = VkDescriptorBufferInfo.callocStack(1, stack)
                .offset(0)
                .range(ubo.getSize().toLong())

            val descriptorWrites = VkWriteDescriptorSet.callocStack(poolObjects.size, stack)

            for (i in 0 until pDescriptorSets.capacity()) {
                val descriptorSet = pDescriptorSets[i]
                bufferInfo.buffer(ubo.getUniformBuffers()[i].buffer())
                poolObjects.forEachIndexed { index, poolObj ->
                    // TODO OPT: maybe group descriptors up by type if that's faster than defining each one by itself
                    val descriptorWrite = descriptorWrites[index]
                        .sType(VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET)
                        .dstBinding(if (poolObj.getBindingLocation() == -1) index else poolObj.getBindingLocation())
                        .dstArrayElement(0)
                        .descriptorType(poolObj.getVkType())
                        .descriptorCount(1)

                    when (poolObj.getVkType()) {
                        VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER -> {
                            descriptorWrite.pBufferInfo(bufferInfo)
                        }

                        VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER -> {
                            if (poolObj is PoolSamplerInfo) {
                                val texture = if (poolObj.samplerIndex == -1) {
                                    TextureManager.BLANK_TEXTURE
                                } else {
                                    currentTextures[poolObj.samplerIndex] ?: TextureManager.BLANK_TEXTURE
                                }

                                val imageInfo = VkDescriptorImageInfo.callocStack(1, stack)
                                    .imageLayout(VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL)
                                    .imageView(texture.textureImage.view)
                                    .sampler(texture.textureSampler!!)

                                descriptorWrite.pImageInfo(imageInfo)
                            }
                        }
                    }
                    descriptorWrite.dstSet(descriptorSet)
                }
                vkUpdateDescriptorSets(device.rawDevice, descriptorWrites, null)
                descriptorSets.setDescriptorPool(descriptorPool)
                descriptorSets.add(descriptorSet)
            }

            ubo.setDescriptors(descriptorSets)
        }
    }

    fun free() {
        vkDestroyDescriptorSetLayout(device.rawDevice, descriptorSetLayout, null)
        vkDestroyDescriptorPool(device.rawDevice, descriptorPool, null)
    }

    interface PoolObjectInfo {
        /**
         * If -1, the object will use the current index in the list when iterating
         * TODO: when converting this to java, make a static variable for -1 and use that
         */
        fun getBindingLocation(): Int
        fun getVkType(): Int
        fun getShaderStage(): Int
    }

    enum class PoolUboInfo : PoolObjectInfo {
        INSTANCE;

        override fun getBindingLocation(): Int {
            return -1
        }

        override fun getVkType(): Int {
            return VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER
        }

        override fun getShaderStage(): Int {
            return VK_SHADER_STAGE_ALL
        }
    }

    data class PoolSamplerInfo(private val bindingLocation: Int, val samplerIndex: Int) : PoolObjectInfo {

        override fun getBindingLocation(): Int {
            return bindingLocation
        }

        override fun getVkType(): Int {
            return VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER
        }

        override fun getShaderStage(): Int {
            return VK_SHADER_STAGE_ALL
        }
    }
}
