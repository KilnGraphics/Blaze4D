package me.hydos.rosella.render.resource

import java.io.InputStream
import java.nio.ByteBuffer

interface Resource {

	val identifier: Identifier

	val loader: ResourceLoader

	fun openStream(): InputStream

	fun readAllBytes(native: Boolean = false): ByteBuffer {
		val bytes = openStream().readBytes()

		if (native) {
			val buffer = ByteBuffer.allocateDirect(bytes.size)
			buffer.put(bytes)
			buffer.rewind()
			return buffer
		}

		return ByteBuffer.wrap(bytes)
	}

	object Empty : Resource {
		override val identifier: Identifier
			get() = TODO("Not yet implemented")
		override val loader: ResourceLoader
			get() = TODO("Not yet implemented")

		override fun openStream(): InputStream {
			TODO("Not yet implemented")
		}
	}
}
