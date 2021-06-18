package me.hydos.rosella.render.vertex

import java.nio.ByteBuffer

class BufferVertexConsumer(val format: VertexFormat, initialCapacity: Int = 256) : VertexConsumer {

	val bufferData = ByteBuffer.allocate(initialCapacity)!!
	private var vertexSize = 0
	private var vertexCount = 0

	override fun pos(x: Float, y: Float, z: Float): VertexConsumer {
		bufferData.putFloat(x)
		bufferData.putFloat(y)
		bufferData.putFloat(z)
		return this
	}

	override fun color(red: Int, green: Int, blue: Int): VertexConsumer {
		bufferData.putFloat(red / 255f)
		bufferData.putFloat(green / 255f)
		bufferData.putFloat(blue / 255f)
		return this
	}

	override fun uv(u: Float, v: Float): VertexConsumer {
		bufferData.putFloat(u)
		bufferData.putFloat(v)
		return this
	}

	override fun nextVertex(): VertexConsumer {
		vertexSize = bufferData.position() / (vertexCount + 1)
		vertexCount++
		return this
	}

	override fun clear() {
		vertexCount = 0
		vertexSize = 0
		bufferData.clear()
	}

	override fun getVertexSize(): Int {
		return vertexSize
	}

	override fun getVertexCount(): Int {
		return vertexCount
	}

	companion object {
		val MAX_BUFFER_SIZE = 2097152
	}
}