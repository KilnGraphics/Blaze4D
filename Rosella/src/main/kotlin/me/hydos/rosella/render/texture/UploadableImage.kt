package me.hydos.rosella.render.texture

import java.nio.ByteBuffer

/**
 * Allows the ability for the software to load the image their own way. especially handy when you generate images
 */
interface UploadableImage {

	fun getWidth(): Int
	fun getHeight(): Int
	fun getChannels(): Int
	fun getPixels(): ByteBuffer
	fun getImageSize(): Int
}