package me.hydos.rosella.render.vertex

import org.lwjgl.vulkan.VK10
import org.lwjgl.vulkan.VkVertexInputAttributeDescription
import org.lwjgl.vulkan.VkVertexInputBindingDescription
import java.util.function.Consumer

/**
 * Used to define the attributes going into a shader.
 */
class VertexFormat(private val elementMap: Map<Int, Element>) {

	private val elements = elementMap.values

	val vkAttributes: VkVertexInputAttributeDescription.Buffer
		get() {
			val attributeDescriptions = VkVertexInputAttributeDescription.callocStack(elementMap.size)

			var offset = 0
			for (entry in elementMap) {
				attributeDescriptions[entry.key]
					.binding(0)
					.location(entry.key)
					.format(entry.value.vkType)
					.offset(offset)
				offset += entry.value.byteLength
			}
			return attributeDescriptions.rewind()
		}

	val vkBindings: VkVertexInputBindingDescription.Buffer
		get() {
			return VkVertexInputBindingDescription.callocStack(1)
				.binding(0)
				.stride(getSize())
				.inputRate(VK10.VK_VERTEX_INPUT_RATE_VERTEX)
		}

	fun getSize(): Int {
		var size = 0
		elements.forEach(Consumer {
			size += it.byteLength
		})
		return size
	}

	enum class DataType(val byteLength: Int) {
		FLOAT(Float.SIZE_BYTES),
		UBYTE(Byte.SIZE_BYTES),
		BYTE(Byte.SIZE_BYTES),
		USHORT(Short.SIZE_BYTES),
		SHORT(Short.SIZE_BYTES),
		UINT(Int.SIZE_BYTES),
		INT(Int.SIZE_BYTES);
	}

	enum class Element(val vkType: Int, val byteLength: Int) {
		POSITION(VK10.VK_FORMAT_R32G32B32_SFLOAT, DataType.FLOAT.byteLength * 3),
		NORMAL(VK10.VK_FORMAT_R32G32B32_SFLOAT, DataType.FLOAT.byteLength * 3),
		COLOR(VK10.VK_FORMAT_R32G32B32_SFLOAT, DataType.FLOAT.byteLength * 3),
		UV(VK10.VK_FORMAT_R32G32_SFLOAT, DataType.FLOAT.byteLength * 2),
		PADDING(VK10.VK_FORMAT_R8_SINT, DataType.BYTE.byteLength),
		GENERIC(VK10.VK_FORMAT_R8_SINT, DataType.BYTE.byteLength),
	}
}