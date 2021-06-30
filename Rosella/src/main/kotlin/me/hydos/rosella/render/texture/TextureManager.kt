package me.hydos.rosella.render.texture

import it.unimi.dsi.fastutil.ints.IntArrayIndirectPriorityQueue
import it.unimi.dsi.fastutil.ints.IntArrayPriorityQueue
import it.unimi.dsi.fastutil.ints.IntPriorityQueue
import me.hydos.rosella.Rosella
import me.hydos.rosella.render.createTextureImage
import me.hydos.rosella.render.createTextureImageView
import me.hydos.rosella.render.device.Device

/**
 * Caches Textures and other texture related objects
 */
class TextureManager(val device: Device) { // TODO: add layers, maybe not in this class but somewhere

	private val textureMap = HashMap<Int, Texture>()
	private val samplerCache = HashMap<SamplerCreateInfo, TextureSampler>() // bro there's like 3 options for this

	private val reusableTexIds = IntArrayPriorityQueue() // may need to eventually swap to longs
	private var largestTexId : Int = 0;

	fun generateTextureId(): Int {
		return if (!reusableTexIds.isEmpty) {
			reusableTexIds.dequeueInt()
		} else {
			largestTexId++
		}
	}

	fun deleteTexture(textureId: Int) {
		textureMap.remove(textureId)
		reusableTexIds.enqueue(textureId)
	}

	fun getTexture(textureId: Int): Texture? {
		return textureMap.get(textureId);
	}

	// TODO: add variant of this method which accepts a pointer or a buffer directly
	// probably wont work if we need to mess with the buffer for the capacity and location and stuff for vulkan
	// for nativeimage, we could also change the format so we don't have to change the channels
	fun uploadTextureToId(
		engine: Rosella,
		textureId: Int,
		image: UploadableImage,
		imgFormat: Int,
		samplerCreateInfo: SamplerCreateInfo
	) {
		val textureImage = TextureImage(0, 0, 0)
		createTextureImage(device, image, engine.renderer, engine.memory, imgFormat, textureImage)
		textureImage.view = createTextureImageView(device, imgFormat, textureImage.textureImage)

		val textureSampler = samplerCache.computeIfAbsent(samplerCreateInfo) {
			TextureSampler(samplerCreateInfo, engine.device)
		}

		textureMap[textureId] = Texture(imgFormat, textureImage, textureSampler.pointer);
	}
}