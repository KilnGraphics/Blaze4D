package me.hydos.rosella.render.vertex

import java.nio.ByteBuffer
import java.util.function.Consumer

class BufferVertexConsumer(val format: VertexFormat) : VertexConsumer {

	var bufferConsumerList = ArrayList<Consumer<ByteBuffer>>()
	private var vertexSize = format.getSize()
	private var vertexCount = 0

	override fun pos(x: Float, y: Float, z: Float): VertexConsumer {
		bufferConsumerList.add(Consumer {
			it.putFloat(x)
			it.putFloat(y)
			it.putFloat(z)
		})
		return this
	}

	override fun color(red: Int, green: Int, blue: Int): VertexConsumer {
		bufferConsumerList.add(Consumer {
			it.putFloat(red / 255f)
			it.putFloat(green / 255f)
			it.putFloat(blue / 255f)
		})
		return this
	}

	override fun uv(u: Float, v: Float): VertexConsumer {
		bufferConsumerList.add(Consumer {
			it.putFloat(u)
			it.putFloat(v)
		})
		return this
	}

	override fun nextVertex(): VertexConsumer {
		vertexCount++
		return this
	}

	override fun clear() {
		bufferConsumerList.clear()
		vertexCount = 0
	}

	override fun getVertexSize(): Int {
		return vertexSize
	}

	override fun getVertexCount(): Int {
		return vertexCount
	}

	companion object {
		const val MAX_BUFFER_SIZE = 2097152
	}
}