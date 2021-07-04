package me.hydos.rosella.render.util.memory

import me.hydos.rosella.Rosella
import me.hydos.rosella.device.VulkanDevice
import me.hydos.rosella.render.renderer.Renderer
import me.hydos.rosella.render.util.ok
import me.hydos.rosella.render.vertex.BufferVertexConsumer
import me.hydos.rosella.render.vertex.VertexConsumer
import me.hydos.rosella.vkobjects.VkCommon
import org.lwjgl.PointerBuffer
import org.lwjgl.system.MemoryStack
import org.lwjgl.system.MemoryStack.stackPush
import org.lwjgl.system.Pointer
import org.lwjgl.util.vma.Vma
import org.lwjgl.util.vma.VmaAllocationCreateInfo
import org.lwjgl.util.vma.VmaAllocatorCreateInfo
import org.lwjgl.util.vma.VmaVulkanFunctions
import org.lwjgl.vulkan.VK10
import org.lwjgl.vulkan.VkBufferCopy
import org.lwjgl.vulkan.VkBufferCreateInfo
import org.lwjgl.vulkan.VkSubmitInfo
import java.nio.ByteBuffer
import java.nio.LongBuffer
import java.util.concurrent.Executors

/**
 * Used for managing CPU and GPU memory.
 * This class will try to handle most vma stuff for the user so they dont have to touch much memory related stuff
 */
class Memory(val common: VkCommon) {

	private val threadCount = 3
	private val executorService = Executors.newFixedThreadPool(threadCount)
	private val mappedMemory = ArrayList<Long>()
	private val buffers = ArrayList<BufferInfo>()

	private val allocator: Long = stackPush().use {
		val vulkanFunctions: VmaVulkanFunctions = VmaVulkanFunctions.callocStack(it)
			.set(common.vkInstance.rawInstance, common.device.rawDevice)

		val createInfo: VmaAllocatorCreateInfo = VmaAllocatorCreateInfo.callocStack(it)
			.physicalDevice(common.device.physicalDevice)
			.device(common.device.rawDevice)
			.pVulkanFunctions(vulkanFunctions)
			.instance(common.vkInstance.rawInstance)
			.vulkanApiVersion(Rosella.VULKAN_VERSION)

		val pAllocator = it.mallocPointer(1)
		Vma.vmaCreateAllocator(createInfo, pAllocator)
		pAllocator[0]
	}

	/**
	 * Maps an allocation with an Pointer Buffer
	 */
	fun map(allocation: Long, unmapOnClose: Boolean, data: PointerBuffer) {
		if (unmapOnClose) {
			mappedMemory.add(allocation)
		}
		Vma.vmaMapMemory(allocator, allocation, data)
	}

	/**
	 * Unmaps allocated memory. this should usually be called on close
	 */
	fun unmap(allocation: Long) {
		executorService.submit {
			Vma.vmaUnmapMemory(allocator, allocation)
		}
	}

	/**
	 * Used for creating the buffer written to before copied to the GPU
	 */
	fun createStagingBuf(
		size: Int,
		pBuffer: LongBuffer,
		stack: MemoryStack,
		callback: (data: PointerBuffer) -> Unit
	): BufferInfo {
		val stagingBuffer: BufferInfo = createBuffer(
			size,
			VK10.VK_BUFFER_USAGE_TRANSFER_SRC_BIT,
			Vma.VMA_MEMORY_USAGE_CPU_ONLY,
			pBuffer
		)
		val data = stack.mallocPointer(1)
		map(stagingBuffer.allocation, true, data)
		callback(data)
		return stagingBuffer
	}


	/**
	 * Used to create a Vulkan Memory Allocator Buffer.
	 */
	fun createBuffer(
		size: Int,
		usage: Int,
		vmaUsage: Int,
		pBuffer: LongBuffer
	): BufferInfo {
		var allocation: Long
		stackPush().use {
			if (size == 0) {
				throw RuntimeException("Failed To Create VMA Buffer Reason: Buffer Is Too Small (0)")
			}

			val vulkanBufferInfo = VkBufferCreateInfo.callocStack(it)
				.sType(VK10.VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO)
				.size(size.toLong())
				.usage(usage)
				.sharingMode(VK10.VK_SHARING_MODE_EXCLUSIVE)

			val vmaBufferInfo: VmaAllocationCreateInfo = VmaAllocationCreateInfo.callocStack(it)
				.usage(vmaUsage)

			val pAllocation = it.mallocPointer(1)
			val result = Vma.vmaCreateBuffer(allocator, vulkanBufferInfo, vmaBufferInfo, pBuffer, pAllocation, null)
			if (result != 0) {
				throw RuntimeException("Failed To Create VMA Buffer. Error Code $result")
			}
			allocation = pAllocation[0]
		}
		return BufferInfo(pBuffer[0], allocation)
	}

	/**
	 * Copies a buffer from one place to another. usually used to copy a staging buffer into GPU mem
	 */
	private fun copyBuffer(srcBuffer: Long, dstBuffer: Long, size: Int, renderer: Renderer, device: VulkanDevice) {
		stackPush().use {
			val pCommandBuffer = it.mallocPointer(1)
			val commandBuffer = renderer.beginCmdBuffer(it, pCommandBuffer)
			run {
				val copyRegion = VkBufferCopy.callocStack(1, it)
				copyRegion.size(size.toLong())
				VK10.vkCmdCopyBuffer(commandBuffer, srcBuffer, dstBuffer, copyRegion)
			}
			VK10.vkEndCommandBuffer(commandBuffer).ok()
			val submitInfo = VkSubmitInfo.callocStack(it)
				.sType(VK10.VK_STRUCTURE_TYPE_SUBMIT_INFO)
				.pCommandBuffers(pCommandBuffer)
			VK10.vkQueueSubmit(renderer.queues.graphicsQueue, submitInfo, VK10.VK_NULL_HANDLE).ok()
			VK10.vkQueueWaitIdle(renderer.queues.graphicsQueue).ok()
			VK10.vkFreeCommandBuffers(device.rawDevice, renderer.commandPool, pCommandBuffer)
		}
	}

	/**
	 * Creates an index buffer from an list of indices
	 */
	fun createIndexBuffer(engine: Rosella, indices: List<Int>): BufferInfo {
		stackPush().use {
			val size: Int = (Integer.BYTES * indices.size)
			val pBuffer = it.mallocLong(1)
			val stagingBuffer = engine.memory.createStagingBuf(size, pBuffer, it) { data ->
				memcpy(data.getByteBuffer(0, size), indices)
			}
			val indexBufferInfo = createBuffer(
				size,
				VK10.VK_BUFFER_USAGE_TRANSFER_DST_BIT or VK10.VK_BUFFER_USAGE_INDEX_BUFFER_BIT,
				Vma.VMA_MEMORY_USAGE_CPU_TO_GPU,
				pBuffer
			)
			val indexBuffer = pBuffer[0]
			copyBuffer(stagingBuffer.buffer, indexBuffer, size, engine.renderer, engine.common.device)
			freeBuffer(stagingBuffer)
			return indexBufferInfo
		}
	}

	/**
	 * Creates a vertex buffer from an List of Vertices
	 */
	fun createVertexBuffer(engine: Rosella, consumer: VertexConsumer): BufferInfo {
		stackPush().use {
			if (consumer is BufferVertexConsumer) {
				val size: Int = consumer.getVertexSize() * consumer.getVertexCount()
				val pBuffer = it.mallocLong(1)
				val stagingBuffer = createStagingBuf(size, pBuffer, it) { data ->
					val dst = data.getByteBuffer(0, size)
					for (bufConsumer in consumer.bufferConsumerList) {
						bufConsumer.accept(dst)
					}
				}
				val vertexBufferInfo = createBuffer(
					size,
					VK10.VK_BUFFER_USAGE_TRANSFER_DST_BIT or VK10.VK_BUFFER_USAGE_VERTEX_BUFFER_BIT,
					Vma.VMA_MEMORY_USAGE_CPU_TO_GPU,
					pBuffer
				)
				val vertexBuffer = pBuffer[0]
				copyBuffer(stagingBuffer.buffer, vertexBuffer, size, engine.renderer, engine.common.device)
				freeBuffer(stagingBuffer)
				return vertexBufferInfo
			} else {
				throw RuntimeException("Cannot handle non buffer based Vertex Consumers")
			}
		}
	}

	/**
	 * Forces a buffer to be freed
	 */
	fun freeBuffer(buffer: BufferInfo) {
		executorService.submit {
			Vma.vmaDestroyBuffer(allocator, buffer.buffer, buffer.allocation)
		}
	}

	/**
	 * Free's all created buffers and mapped memory
	 */
	fun free() {
		for (memory in mappedMemory) {
			unmap(memory)
		}
		for (buffer in buffers) {
			freeBuffer(buffer)
		}
		Vma.vmaDestroyAllocator(allocator)
	}
}

data class BufferInfo(val buffer: Long, val allocation: Long)

/**
 * Copies indices into the specified buffer
 */
fun memcpy(buffer: ByteBuffer, indices: List<Int>) {
	for (index in indices) {
		buffer.putInt(index)
	}
}

/**
 * Copies an ByteBuffer into another ByteBuffer
 */
fun memcpy(dst: ByteBuffer, src: ByteBuffer, size: Long) {
	src.limit(size.toInt())
	dst.put(src)
	src.limit(src.capacity()).rewind()
}

fun List<Pointer>.asPointerBuffer(): PointerBuffer {
	val buffer = MemoryStack.stackGet().mallocPointer(size)

	for (pointer in this) {
		buffer.put(pointer)
	}

	return buffer.rewind()
}
