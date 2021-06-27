package me.hydos.rosella.render.vertex

import java.nio.ByteBuffer
import java.util.*
import java.util.function.Consumer

class BufferVertexConsumer(override val format: VertexFormat) : VertexConsumer {

	var bufferConsumerList : MutableList<Consumer<ByteBuffer>> = ArrayList()
	private var vertexSize = format.getSize()
	private var vertexCount = 0

	private var debugSize = 0

	override fun pos(x: Float, y: Float, z: Float): VertexConsumer {
		bufferConsumerList.add(Consumer {
			it.putFloat(x)
			it.putFloat(y)
			it.putFloat(z)
		})
		debugSize += 3 * Float.SIZE_BYTES
		return this
	}

	override fun color(red: Int, green: Int, blue: Int): VertexConsumer {
		bufferConsumerList.add(Consumer {
			it.putFloat(red / 255f)
			it.putFloat(green / 255f)
			it.putFloat(blue / 255f)
		})

		debugSize += 3 * Float.SIZE_BYTES
		return this
	}

	override fun normal(x: Float, y: Float, z: Float): VertexConsumer {
		bufferConsumerList.add(Consumer {
			it.putFloat(x)
			it.putFloat(y)
			it.putFloat(z)
		})

		debugSize += 3 * Float.SIZE_BYTES
		return this
	}

	override fun color(red: Int, green: Int, blue: Int, alpha: Int): VertexConsumer {
		bufferConsumerList.add(Consumer {
			it.putFloat(red / 255f)
			it.putFloat(green / 255f)
			it.putFloat(blue / 255f)
			it.putFloat(alpha / 255f)
		})

		debugSize += 4 * Float.SIZE_BYTES
		return this
	}

	override fun uv(u: Float, v: Float): VertexConsumer {
		bufferConsumerList.add(Consumer {
			it.putFloat(u)
			it.putFloat(v)
		})
		debugSize += 2 * Float.SIZE_BYTES
		return this
	}

	override fun uv(u: Short, v: Short): VertexConsumer {
		bufferConsumerList.add(Consumer {
			it.putShort(u)
			it.putShort(v)
		})

		debugSize += 2 * Short.SIZE_BYTES
		return this
	}

	override fun nextVertex(): VertexConsumer {
		if (debugSize != vertexSize) {
			throw RuntimeException("Incorrect vertex size passed. Received $debugSize but wanted $vertexSize")
		}

		debugSize = 0
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

	override fun copy(): VertexConsumer {
		val consumer = BufferVertexConsumer(format)
		consumer.debugSize = this.debugSize
		consumer.bufferConsumerList = Collections.unmodifiableList(this.bufferConsumerList)
		consumer.vertexCount = this.vertexCount
		consumer.vertexSize = this.vertexSize
		return consumer
	}
}