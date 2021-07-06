package me.hydos.rosella.render.renderer

import me.hydos.rosella.Rosella
import me.hydos.rosella.device.VulkanDevice
import me.hydos.rosella.device.VulkanQueues
import me.hydos.rosella.display.Display
import me.hydos.rosella.memory.Memory
import me.hydos.rosella.render.*
import me.hydos.rosella.render.info.InstanceInfo
import me.hydos.rosella.render.info.RenderInfo
import me.hydos.rosella.render.shader.ShaderProgram
import me.hydos.rosella.render.swapchain.DepthBuffer
import me.hydos.rosella.render.swapchain.Frame
import me.hydos.rosella.render.swapchain.RenderPass
import me.hydos.rosella.render.swapchain.Swapchain
import me.hydos.rosella.render.util.ok
import me.hydos.rosella.scene.`object`.impl.SimpleObjectManager
import me.hydos.rosella.vkobjects.VkCommon
import org.lwjgl.PointerBuffer
import org.lwjgl.system.MemoryStack
import org.lwjgl.vulkan.*
import org.lwjgl.vulkan.VK10.*

class Renderer(val common: VkCommon, display: Display, rosella: Rosella) {

	var depthBuffer = DepthBuffer()

	lateinit var inFlightFrames: MutableList<Frame>
	private lateinit var imagesInFlight: MutableMap<Int, Frame>
	private var currentFrame = 0

	private var resizeFramebuffer: Boolean = false

	private var r: Float = 0.2f
	private var g: Float = 0.2f
	private var b: Float = 0.2f

	lateinit var swapchain: Swapchain
	lateinit var renderPass: RenderPass

	var queues: VulkanQueues = VulkanQueues(common)

	var commandPool: Long = 0
	lateinit var commandBuffers: ArrayList<VkCommandBuffer>

	init {
		createCmdPool(common.device, this, common.surface)
		createSwapChain(common, display, rosella.objectManager as SimpleObjectManager)
	}

	private fun createSwapChain(common: VkCommon, display: Display, objectManager: SimpleObjectManager) {
		this.swapchain = Swapchain(display, common.device.rawDevice, common.device.physicalDevice, common.surface)
		this.renderPass = RenderPass(common.device, swapchain, this)
		createImgViews(swapchain, common.device)
		for (material in objectManager.materials) {
			material.pipeline = objectManager.pipelineManager.getPipeline(material, this)
		}
		depthBuffer.createDepthResources(common.device, swapchain, this)
		createFrameBuffers()
//		engine.camera.createViewAndProj(swapchain)
		rebuildCommandBuffers(renderPass, objectManager)
		createSyncObjects()
	}

	fun beginCmdBuffer(stack: MemoryStack, pCommandBuffer: PointerBuffer, device: VulkanDevice): VkCommandBuffer {
		val allocInfo = VkCommandBufferAllocateInfo.callocStack(stack)
				.sType(VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO)
				.level(VK_COMMAND_BUFFER_LEVEL_PRIMARY)
				.commandPool(commandPool)
				.commandBufferCount(1)
		vkAllocateCommandBuffers(device.rawDevice, allocInfo, pCommandBuffer).ok()
		val commandBuffer = VkCommandBuffer(pCommandBuffer[0], device.rawDevice)
		val beginInfo = VkCommandBufferBeginInfo.callocStack(stack)
				.sType(VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO)
				.flags(VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT)
		vkBeginCommandBuffer(commandBuffer, beginInfo).ok()
		return commandBuffer
	}

	fun render(rosella: Rosella) {
		MemoryStack.stackPush().use { stack ->
			val thisFrame = inFlightFrames[currentFrame]
			vkWaitForFences(rosella.common.device.rawDevice, thisFrame.pFence(), true, UINT64_MAX).ok()

			val pImageIndex = stack.mallocInt(1)

			var vkResult: Int = KHRSwapchain.vkAcquireNextImageKHR(
					rosella.common.device.rawDevice,
					swapchain.swapChain,
					UINT64_MAX,
					thisFrame.imageAvailableSemaphore(),
					VK_NULL_HANDLE,
					pImageIndex
			)

			if (vkResult == KHRSwapchain.VK_ERROR_OUT_OF_DATE_KHR) {
				recreateSwapChain(rosella.common.display, rosella)
				return
			}

			val imageIndex = pImageIndex[0]

			for (shader in (rosella.objectManager as SimpleObjectManager).shaderManager.cachedShaders.keys) {
				shader.updateUbos(imageIndex, swapchain, rosella.objectManager)
			}

			if (imagesInFlight.containsKey(imageIndex)) {
				vkWaitForFences(
						rosella.common.device.rawDevice,
						imagesInFlight[imageIndex]!!.fence(),
						true,
						UINT64_MAX
				).ok()
			}
			imagesInFlight[imageIndex] = thisFrame
			val submitInfo = VkSubmitInfo.callocStack(stack)
					.sType(VK_STRUCTURE_TYPE_SUBMIT_INFO)
					.waitSemaphoreCount(1)
					.pWaitSemaphores(thisFrame.pImageAvailableSemaphore())
					.pWaitDstStageMask(stack.ints(VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT))
					.pSignalSemaphores(thisFrame.pRenderFinishedSemaphore())
					.pCommandBuffers(stack.pointers(commandBuffers[imageIndex]))
			vkResetFences(rosella.common.device.rawDevice, thisFrame.pFence()).ok()
			vkQueueSubmit(queues.graphicsQueue, submitInfo, thisFrame.fence()).ok()

			val presentInfo = VkPresentInfoKHR.callocStack(stack)
					.sType(KHRSwapchain.VK_STRUCTURE_TYPE_PRESENT_INFO_KHR)
					.pWaitSemaphores(thisFrame.pRenderFinishedSemaphore())
					.swapchainCount(1)
					.pSwapchains(stack.longs(swapchain.swapChain))
					.pImageIndices(pImageIndex)

			vkResult = KHRSwapchain.vkQueuePresentKHR(queues.presentQueue, presentInfo)

			if (vkResult == KHRSwapchain.VK_ERROR_OUT_OF_DATE_KHR || vkResult == KHRSwapchain.VK_SUBOPTIMAL_KHR || resizeFramebuffer) {
				resizeFramebuffer = false
				recreateSwapChain(rosella.common.display, rosella)
				rosella.objectManager.pipelineManager.invalidatePipelines(swapchain, rosella)
			} else if (vkResult != VK_SUCCESS) {
				throw RuntimeException("Failed to present swap chain image")
			}

			currentFrame = (currentFrame + 1) % MAX_FRAMES_IN_FLIGHT
		}
	}

	private fun recreateSwapChain(window: Display, rosella: Rosella) {
		MemoryStack.stackPush().use { stack ->
			val width = stack.ints(0)
			val height = stack.ints(0)
			while (width[0] == 0 && height[0] == 0) {
				window.waitForNonZeroSize()
			}
		}

		vkDeviceWaitIdle(rosella.common.device.rawDevice).ok()
		freeSwapChain(rosella)
		createSwapChain(rosella.common, window, rosella.objectManager as SimpleObjectManager)
//		camera.createViewAndProj(swapchain)
	}

	fun freeSwapChain(rosella: Rosella) {
		for (shaderPair in (rosella.objectManager as SimpleObjectManager).shaderManager.cachedShaders.keys) {
			vkDestroyDescriptorPool(rosella.common.device.rawDevice, shaderPair.descriptorPool, null)
		}

		clearCommandBuffers(rosella.common.device)

		// Free Depth Buffer
		depthBuffer.free(rosella.common.device)

		swapchain.frameBuffers.forEach { framebuffer ->
			vkDestroyFramebuffer(
					rosella.common.device.rawDevice,
					framebuffer,
					null
			)
		}
		vkDestroyRenderPass(rosella.common.device.rawDevice, renderPass.renderPass, null)
		swapchain.swapChainImageViews.forEach { imageView ->
			vkDestroyImageView(
					rosella.common.device.rawDevice,
					imageView,
					null
			)
		}

		swapchain.free(rosella.common.device.rawDevice)
	}

	fun clearCommandBuffers(device: VulkanDevice) {
		if (commandBuffers.size != 0) {
			vkFreeCommandBuffers(device.rawDevice, commandPool, Memory.asPointerBuffer(commandBuffers))
			commandBuffers.clear()
		}
	}

	private fun createSyncObjects() {
		inFlightFrames = ArrayList(MAX_FRAMES_IN_FLIGHT)
		imagesInFlight = HashMap(swapchain.swapChainImages.size)

		MemoryStack.stackPush().use { stack ->
			val semaphoreInfo = VkSemaphoreCreateInfo.callocStack(stack)
			semaphoreInfo.sType(VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO)
			val fenceInfo = VkFenceCreateInfo.callocStack(stack)
			fenceInfo.sType(VK_STRUCTURE_TYPE_FENCE_CREATE_INFO)
			fenceInfo.flags(VK_FENCE_CREATE_SIGNALED_BIT)
			val pImageAvailableSemaphore = stack.mallocLong(1)
			val pRenderFinishedSemaphore = stack.mallocLong(1)
			val pFence = stack.mallocLong(1)
			for (i in 0 until MAX_FRAMES_IN_FLIGHT) {
				vkCreateSemaphore(
						common.device.rawDevice,
						semaphoreInfo,
						null,
						pImageAvailableSemaphore
				).ok()
				vkCreateSemaphore(
						common.device.rawDevice,
						semaphoreInfo,
						null,
						pRenderFinishedSemaphore
				).ok()
				vkCreateFence(common.device.rawDevice, fenceInfo, null, pFence).ok()
				inFlightFrames.add(
						Frame(
								pImageAvailableSemaphore[0],
								pRenderFinishedSemaphore[0],
								pFence[0]
						)
				)
			}
		}
	}

	fun windowResizeCallback(width: Int, height: Int) {
		this.resizeFramebuffer = true
	}

	private fun createFrameBuffers() {
		swapchain.frameBuffers = ArrayList(swapchain.swapChainImageViews.size)
		MemoryStack.stackPush().use { stack ->
			val attachments = stack.longs(VK_NULL_HANDLE, depthBuffer.depthImageView)
			val pFramebuffer = stack.mallocLong(1)
			val framebufferInfo = VkFramebufferCreateInfo.callocStack(stack)
					.sType(VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO)
					.renderPass(renderPass.renderPass)
					.width(swapchain.swapChainExtent.width())
					.height(swapchain.swapChainExtent.height())
					.layers(1)
			for (imageView in swapchain.swapChainImageViews) {
				attachments.put(0, imageView)
				framebufferInfo.pAttachments(attachments)
				vkCreateFramebuffer(common.device.rawDevice, framebufferInfo, null, pFramebuffer).ok()
				swapchain.frameBuffers.add(pFramebuffer[0])
			}
		}
	}

	/**
	 * Create the Command Buffers
	 */
	fun rebuildCommandBuffers(renderPass: RenderPass, rosella: SimpleObjectManager) {
		rosella.rebuildCmdBuffers(renderPass, null, null) //TODO: move it into here
		val usedShaders = ArrayList<ShaderProgram>()
		for (material in rosella.materials) {
			if (!usedShaders.contains(material.shader)) {
				usedShaders.add(material.shader!!)
			}
		}

		for (instances in rosella.renderObjects.values) {
			for (instance in instances) {
				instance.rebuild(this)
			}
		}

		MemoryStack.stackPush().use {
			val commandBuffersCount: Int = swapchain.frameBuffers.size

			commandBuffers = ArrayList(commandBuffersCount)

			val pCommandBuffers = allocateCmdBuffers(
					it,
					common.device,
					commandPool,
					commandBuffersCount
			)

			for (i in 0 until commandBuffersCount) {
				commandBuffers.add(
						VkCommandBuffer(
								pCommandBuffers[i],
								common.device.rawDevice
						)
				)
			}

			val beginInfo = createBeginInfo(it)
			val renderPassInfo = createRenderPassInfo(it, renderPass)
			val renderArea = createRenderArea(it, 0, 0, swapchain)
			val clearValues = createClearValues(it, r, g, b, 1.0f, 0)

			renderPassInfo.renderArea(renderArea)
					.pClearValues(clearValues)

			for (i in 0 until commandBuffersCount) {
				val commandBuffer = commandBuffers[i]
				vkBeginCommandBuffer(commandBuffer, beginInfo).ok()
				renderPassInfo.framebuffer(swapchain.frameBuffers[i])

				vkCmdBeginRenderPass(commandBuffer, renderPassInfo, VK_SUBPASS_CONTENTS_INLINE)
				for (renderInfo in rosella.renderObjects.keys) {
					bindRenderInfo(renderInfo, it, commandBuffer)
					for (instance in rosella.renderObjects[renderInfo]!!) {
						bindInstanceInfo(instance, it, commandBuffer, i)
						vkCmdDrawIndexed(commandBuffer, renderInfo.indicesSize, 1, 0, 0, 0)
					}
				}
				vkCmdEndRenderPass(commandBuffer)
				vkEndCommandBuffer(commandBuffer).ok()
			}
		}
	}

	private fun bindRenderInfo(
			renderInfo: RenderInfo,
			stack: MemoryStack,
			commandBuffer: VkCommandBuffer
	) {
		val offsets = stack.longs(0)
		val vertexBuffers = stack.longs(renderInfo.vertexBuffer.buffer())
		vkCmdBindVertexBuffers(commandBuffer, 0, vertexBuffers, offsets)
		vkCmdBindIndexBuffer(commandBuffer, renderInfo.indexBuffer.buffer(), 0, VK_INDEX_TYPE_UINT32)
	}

	private fun bindInstanceInfo(
			instanceInfo: InstanceInfo,
			matrix: MemoryStack,
			commandBuffer: VkCommandBuffer,
			commandBufferIndex: Int
	) {
		vkCmdBindPipeline(
				commandBuffer,
				VK_PIPELINE_BIND_POINT_GRAPHICS,
				instanceInfo.material.pipeline.graphicsPipeline
		)

		vkCmdBindDescriptorSets(
				commandBuffer,
				VK_PIPELINE_BIND_POINT_GRAPHICS,
				instanceInfo.material.pipeline.pipelineLayout,
				0,
				matrix.longs(instanceInfo.ubo.getDescriptors().descriptorSets[commandBufferIndex]),
				null
		)
	}

	fun clearColor(red: Float, green: Float, blue: Float, rosella: Rosella) {
		if (this.r != red || this.g != green || this.b != blue) {
			this.r = red
			this.g = green
			this.b = blue
			rebuildCommandBuffers(renderPass, rosella.objectManager as SimpleObjectManager)
		}
	}

	companion object {
		const val MAX_FRAMES_IN_FLIGHT = 2
		const val UINT64_MAX = -0x1L
	}
}
