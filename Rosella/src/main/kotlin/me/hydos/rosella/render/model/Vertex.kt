package me.hydos.rosella.render.model

import org.joml.Vector2fc
import org.joml.Vector3fc
import org.lwjgl.vulkan.VK10.*
import org.lwjgl.vulkan.VkVertexInputAttributeDescription
import org.lwjgl.vulkan.VkVertexInputBindingDescription


class Vertex(val pos: Vector3fc, val color: Vector3fc, val texCoords: Vector2fc) {
	companion object {
		const val SIZEOF = (3 + 3 + 2) * java.lang.Float.BYTES

		internal val bindingDescription: VkVertexInputBindingDescription.Buffer
			get() {
				val bindingDescription = VkVertexInputBindingDescription.callocStack(1)
				bindingDescription.binding(0)
				bindingDescription.stride(SIZEOF)
				bindingDescription.inputRate(VK_VERTEX_INPUT_RATE_VERTEX)
				return bindingDescription
			}

		internal val attributeDescriptions: VkVertexInputAttributeDescription.Buffer
			get() {
				val attributeDescriptions = VkVertexInputAttributeDescription.callocStack(3)
				// Pos
				attributeDescriptions[0]
					.binding(0)
					.location(0)
					.format(VK_FORMAT_R32G32B32_SFLOAT)
					.offset(0) // Offset

				// Colour
				attributeDescriptions[1]
					.binding(0)
					.location(1)
					.format(VK_FORMAT_R32G32B32_SFLOAT)
					.offset(3 * java.lang.Float.BYTES) // Offset

				// Tex Coords
				attributeDescriptions[2]
					.binding(0)
					.location(2)
					.format(VK_FORMAT_R32G32_SFLOAT)
					.offset((3 + 3) * java.lang.Float.BYTES) // Offset

				return attributeDescriptions.rewind()
			}
	}
}