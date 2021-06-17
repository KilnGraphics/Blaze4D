package me.hydos.rosella.render.renderer

import me.hydos.rosella.Rosella
import me.hydos.rosella.render.*
import me.hydos.rosella.render.camera.Camera
import me.hydos.rosella.render.device.Device
import me.hydos.rosella.render.device.Queues
import me.hydos.rosella.render.io.JUnit
import me.hydos.rosella.render.io.Window
import me.hydos.rosella.render.model.Renderable
import me.hydos.rosella.render.shader.ShaderProgram
import me.hydos.rosella.render.swapchain.DepthBuffer
import me.hydos.rosella.render.swapchain.Frame
import me.hydos.rosella.render.swapchain.RenderPass
import me.hydos.rosella.render.swapchain.SwapChain
import me.hydos.rosella.render.util.memory.asPointerBuffer
import me.hydos.rosella.render.util.ok
import org.lwjgl.PointerBuffer
import org.lwjgl.glfw.GLFW
import org.lwjgl.system.MemoryStack
import org.lwjgl.vulkan.*
import org.lwjgl.vulkan.VK10.*

class Renderer {

	var depthBuffer = DepthBuffer()

	lateinit var inFlightFrames: MutableList<Frame>
	lateinit var imagesInFlight: MutableMap<Int, Frame>
	private var currentFrame = 0

	private var resizeFramebuffer: Boolean = false

	private var r: Float = 0f
	private var g: Float = 0f
	private var b: Float = 0f

	lateinit var swapChain: SwapChain
	lateinit var renderPass: RenderPass

	lateinit var device: Device

	var queues: Queues = Queues()

	var commandPool: Long = 0
	lateinit var commandBuffers: ArrayList<VkCommandBuffer>

	var safeQueue = ArrayList<JUnit>()

	private fun createSwapChain(engine: Rosella) {
		this.swapChain = SwapChain(engine, device.device, device.physicalDevice, engine.surface)
		this.renderPass = RenderPass(device, swapChain, engine)
		createImgViews(swapChain, device)
		for (material in engine.materials.values) {
			material.createPipeline(
				device,
				swapChain,
				renderPass,
				material.shader.raw.descriptorSetLayout,
				engine.polygonMode
			)
		}
		depthBuffer.createDepthResources(device, swapChain, this)
		createFrameBuffers()
		engine.camera.createViewAndProj(swapChain)
		rebuildCommandBuffers(renderPass, engine)
		createSyncObjects()
	}

	fun beginCmdBuffer(stack: MemoryStack, pCommandBuffer: PointerBuffer): VkCommandBuffer {
		val allocInfo = VkCommandBufferAllocateInfo.callocStack(stack)
			.sType(VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO)
			.level(VK_COMMAND_BUFFER_LEVEL_PRIMARY)
			.commandPool(commandPool)
			.commandBufferCount(1)
		vkAllocateCommandBuffers(device.device, allocInfo, pCommandBuffer)
		val commandBuffer = VkCommandBuffer(pCommandBuffer[0], device.device)
		val beginInfo = VkCommandBufferBeginInfo.callocStack(stack)
			.sType(VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO)
			.flags(VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT)
		vkBeginCommandBuffer(commandBuffer, beginInfo)
		return commandBuffer
	}

	fun render(engine: Rosella) {
		MemoryStack.stackPush().use { stack ->
			val thisFrame = inFlightFrames[currentFrame]
			vkWaitForFences(device.device, thisFrame.pFence(), true, UINT64_MAX)

			for (jUnit in safeQueue) {
				jUnit.run()
			}
			safeQueue.clear()

			val pImageIndex = stack.mallocInt(1)

			var vkResult: Int = KHRSwapchain.vkAcquireNextImageKHR(
				device.device,
				swapChain.swapChain,
				UINT64_MAX,
				thisFrame.imageAvailableSemaphore(),
				VK_NULL_HANDLE,
				pImageIndex
			)

			if (vkResult == KHRSwapchain.VK_ERROR_OUT_OF_DATE_KHR) {
				recreateSwapChain(engine.window, engine.camera, engine)
				return
			}

			val imageIndex = pImageIndex[0]

			for (shader in engine.shaderManager.shaders.values) {
				shader.updateUbos(imageIndex, swapChain, engine)
			}

			if (imagesInFlight.containsKey(imageIndex)) {
				vkWaitForFences(device.device, imagesInFlight[imageIndex]!!.fence(), true, UINT64_MAX)
			}
			imagesInFlight[imageIndex] = thisFrame
			val submitInfo = VkSubmitInfo.callocStack(stack)
				.sType(VK_STRUCTURE_TYPE_SUBMIT_INFO)
				.waitSemaphoreCount(1)
				.pWaitSemaphores(thisFrame.pImageAvailableSemaphore())
				.pWaitDstStageMask(stack.ints(VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT))
				.pSignalSemaphores(thisFrame.pRenderFinishedSemaphore())
				.pCommandBuffers(stack.pointers(commandBuffers[imageIndex]))
			vkResetFences(device.device, thisFrame.pFence())
			vkQueueSubmit(queues.graphicsQueue, submitInfo, thisFrame.fence()).ok()

			val presentInfo = VkPresentInfoKHR.callocStack(stack)
				.sType(KHRSwapchain.VK_STRUCTURE_TYPE_PRESENT_INFO_KHR)
				.pWaitSemaphores(thisFrame.pRenderFinishedSemaphore())
				.swapchainCount(1)
				.pSwapchains(stack.longs(swapChain.swapChain))
				.pImageIndices(pImageIndex)

			vkResult = KHRSwapchain.vkQueuePresentKHR(queues.presentQueue, presentInfo)

			if (vkResult == KHRSwapchain.VK_ERROR_OUT_OF_DATE_KHR || vkResult == KHRSwapchain.VK_SUBOPTIMAL_KHR || resizeFramebuffer) {
				resizeFramebuffer = false
				recreateSwapChain(engine.window, engine.camera, engine)
			} else if (vkResult != VK_SUCCESS) {
				throw RuntimeException("Failed to present swap chain image")
			}

			currentFrame = (currentFrame + 1) % MAX_FRAMES_IN_FLIGHT
		}
	}

	fun recreateSwapChain(window: Window, camera: Camera, engine: Rosella) {
		MemoryStack.stackPush().use { stack ->
			val width = stack.ints(0)
			val height = stack.ints(0)
			while (width[0] == 0 && height[0] == 0) {
				GLFW.glfwGetFramebufferSize(window.windowPtr, width, height)
//				GLFW.glfwWaitEvents()
				println("Remember to uncomment this")
			}
		}

		vkDeviceWaitIdle(device.device)
		freeSwapChain(engine)
		createSwapChain(engine)
		camera.createViewAndProj(swapChain)
	}

	fun freeSwapChain(engine: Rosella) {
		for (shaderPair in engine.shaderManager.shaders.values) {
			vkDestroyDescriptorPool(device.device, shaderPair.descriptorPool, null)
		}

		vkFreeCommandBuffers(device.device, commandPool, commandBuffers.asPointerBuffer())

		for (material in engine.materials.values) {
			material.free(device)
		}

		// Free Depth Buffer
		depthBuffer.free(device)

		swapChain.frameBuffers.forEach { framebuffer ->
			vkDestroyFramebuffer(
				device.device,
				framebuffer,
				null
			)
		}
		vkDestroyRenderPass(device.device, renderPass.renderPass, null)
		swapChain.swapChainImageViews.forEach { imageView -> vkDestroyImageView(device.device, imageView, null) }

		swapChain.free(engine.device.device)
	}

	private fun createSyncObjects() {
		inFlightFrames = ArrayList(MAX_FRAMES_IN_FLIGHT)
		imagesInFlight = HashMap(swapChain.swapChainImages.size)

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
					device.device,
					semaphoreInfo,
					null,
					pImageAvailableSemaphore
				).ok()
				vkCreateSemaphore(
					device.device,
					semaphoreInfo,
					null,
					pRenderFinishedSemaphore
				).ok()
				vkCreateFence(device.device, fenceInfo, null, pFence).ok()
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
		swapChain.frameBuffers = ArrayList(swapChain.swapChainImageViews.size)
		MemoryStack.stackPush().use { stack ->
			val attachments = stack.longs(VK_NULL_HANDLE, depthBuffer.depthImageView)
			val pFramebuffer = stack.mallocLong(1)
			val framebufferInfo = VkFramebufferCreateInfo.callocStack(stack)
				.sType(VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO)
				.renderPass(renderPass.renderPass)
				.width(swapChain.swapChainExtent.width())
				.height(swapChain.swapChainExtent.height())
				.layers(1)
			for (imageView in swapChain.swapChainImageViews) {
				attachments.put(0, imageView)
				framebufferInfo.pAttachments(attachments)
				vkCreateFramebuffer(device.device, framebufferInfo, null, pFramebuffer).ok()
				swapChain.frameBuffers.add(pFramebuffer[0])
			}
		}
	}

	/**
	 * Create the Command Buffers
	 * TODO: instancing
	 */
	fun rebuildCommandBuffers(renderPass: RenderPass, engine: Rosella) {
		val usedShaders = ArrayList<ShaderProgram>()
		for (material in engine.materials.values) {
			if (!usedShaders.contains(material.shader)) {
				usedShaders.add(material.shader)
			}
		}

		for (shader in usedShaders) {
			shader.raw.createPool(swapChain)
		}

		for (renderObject in engine.renderObjects.values) {
			renderObject.resize(this)
		}

		MemoryStack.stackPush().use {
			val commandBuffersCount: Int = swapChain.frameBuffers.size

			commandBuffers = ArrayList(commandBuffersCount)

			val pCommandBuffers = allocateCmdBuffers(
				it,
				device,
				commandPool,
				commandBuffersCount
			)

			for (i in 0 until commandBuffersCount) {
				commandBuffers.add(
					VkCommandBuffer(
						pCommandBuffers[i],
						device.device
					)
				)
			}

			val beginInfo = createBeginInfo(it)
			val renderPassInfo = createRenderPassInfo(it, renderPass)
			val renderArea = createRenderArea(it, 0, 0, swapChain)
			val clearValues = createClearValues(it, r, g, b, 1.0f, 0)

			renderPassInfo.renderArea(renderArea)
				.pClearValues(clearValues)

			for (i in 0 until commandBuffersCount) {
				val commandBuffer = commandBuffers[i]
				vkBeginCommandBuffer(commandBuffer, beginInfo).ok()
				renderPassInfo.framebuffer(swapChain.frameBuffers[i])

				vkCmdBeginRenderPass(commandBuffer, renderPassInfo, VK_SUBPASS_CONTENTS_INLINE)
				run {
					for (renderObject in engine.renderObjects.values) {
						bindModel(renderObject, it, renderObject.getDescriptorSets()[i], commandBuffer)
						vkCmdDrawIndexed(commandBuffer, renderObject.getIndices().size, 1, 0, 0, 0)
					}
				}
				vkCmdEndRenderPass(commandBuffer)
				vkEndCommandBuffer(commandBuffer).ok()
			}
		}
		println("Command Buffers rebuilt! " + engine.renderObjects.size + " objects rendered.")
	}

	private fun bindModel(
		renderObject: Renderable,
		matrix: MemoryStack,
		descriptorSet: Long,
		commandBuffer: VkCommandBuffer
	) {
		vkCmdBindPipeline(
			commandBuffer,
			VK_PIPELINE_BIND_POINT_GRAPHICS,
			renderObject.getMaterial().graphicsPipeline
		)

		val offsets = matrix.longs(0)
		val vertexBuffers = matrix.longs(renderObject.getVerticesBuffer())
		vkCmdBindVertexBuffers(commandBuffer, 0, vertexBuffers, offsets)
		vkCmdBindIndexBuffer(commandBuffer, renderObject.getIndicesBuffer(), 0, VK_INDEX_TYPE_UINT32)
		vkCmdBindDescriptorSets(
			commandBuffer,
			VK_PIPELINE_BIND_POINT_GRAPHICS,
			renderObject.getMaterial().pipelineLayout,
			0,
			matrix.longs(descriptorSet),
			null
		)
	}

	/**
	 * Called after the vulkan device and instance have been initialized.
	 */
	fun initialize(engine: Rosella) {
		device = engine.device
		createCmdPool(this, engine.surface)
		createSwapChain(engine)
	}

	fun clearColor(red: Float, green: Float, blue: Float, rosella: Rosella) {
		if (this.r != red || this.g != green || this.b != blue) {
			this.r = red
			this.g = green
			this.b = blue
			rebuildCommandBuffers(renderPass, rosella)
		}
	}

	companion object {
		const val MAX_FRAMES_IN_FLIGHT = 2
		const val UINT64_MAX = -0x1L
	}
}
