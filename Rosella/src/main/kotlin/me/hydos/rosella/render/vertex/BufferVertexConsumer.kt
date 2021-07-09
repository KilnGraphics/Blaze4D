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

	override fun color(red: Byte, green: Byte, blue: Byte): VertexConsumer {
		bufferConsumerList.add(Consumer {
			it.put(red)
			it.put(green)
			it.put(blue)
		})

		debugSize += 3 * Byte.SIZE_BYTES
		return this
	}

	override fun color(red: Byte, green: Byte, blue: Byte, alpha: Byte): VertexConsumer {
		bufferConsumerList.add(Consumer {
			it.put(red)
			it.put(green)
			it.put(blue)
			it.put(alpha)
		})

		debugSize += 4 * Byte.SIZE_BYTES
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

	fun putByte(index: Int, value: Byte): VertexConsumer {
		bufferConsumerList.add(Consumer {
			it.put(index, value)
		})

		debugSize += Byte.SIZE_BYTES
		return this;
	}

	fun putShort(index: Int, value: Short): VertexConsumer {
		bufferConsumerList.add(Consumer {
			it.putShort(index, value)
		})

		debugSize += Short.SIZE_BYTES
		return this;
	}

	fun putFloat(index: Int, value: Float): VertexConsumer {
		bufferConsumerList.add(Consumer {
			it.putFloat(index, value)
		})

		debugSize += Float.SIZE_BYTES
		return this;
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

	override fun hashCode(): Int {
		return Objects.hash(bufferConsumerList, vertexSize, format)
	}

	override fun equals(other: Any?): Boolean {
		if (this === other) return true
		if (javaClass != other?.javaClass) return false

		other as BufferVertexConsumer

		if (format != other.format) return false
		if (bufferConsumerList != other.bufferConsumerList) return false
		if (vertexSize != other.vertexSize) return false

		return true
	}

}