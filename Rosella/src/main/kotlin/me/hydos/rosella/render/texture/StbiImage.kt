package me.hydos.rosella.render.texture

import me.hydos.rosella.render.resource.Resource
import org.lwjgl.stb.STBImage
import org.lwjgl.system.MemoryStack
import java.nio.ByteBuffer

class StbiImage(resource: Resource) : UploadableImage {

	private var height: Int
	private var width: Int
	private var channels: Int
	private var pixelSize: Int
	private var pixels: ByteBuffer

	init {
		MemoryStack.stackPush().use { stack ->
			val file = resource.readAllBytes(true)
			val pWidth = stack.mallocInt(1)
			val pHeight = stack.mallocInt(1)
			val pChannels = stack.mallocInt(1)
			var pixels: ByteBuffer? =
				STBImage.stbi_load_from_memory(file, pWidth, pHeight, pChannels, STBImage.STBI_rgb_alpha)
			if (pixels == null) {
				pixels = ByteBuffer.wrap(resource.openStream().readAllBytes())
				if (pixels == null) {
					throw RuntimeException("Failed to load texture image ${resource.identifier}")
				}
			}

			this.width = pWidth[0]
			this.height = pHeight[0]
			this.channels = pChannels[0]
			this.pixels = pixels
			this.pixelSize = 4 // ARGB = 4?
		}
	}


	override fun getWidth(): Int {
		return width
	}

	override fun getHeight(): Int {
		return height
	}

	override fun getChannels(): Int {
		return channels
	}

	override fun getBytesPerPixel(): Int {
		return pixelSize
	}

	override fun getPixels(region: ImageRegion): ByteBuffer {
		return pixels //FIXME use image size
	}
}