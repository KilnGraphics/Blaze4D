package me.hydos.rosella.render.texture

import it.unimi.dsi.fastutil.ints.IntArrayPriorityQueue
import it.unimi.dsi.fastutil.ints.IntPriorityQueues
import me.hydos.rosella.Rosella
import me.hydos.rosella.render.createTextureImage
import me.hydos.rosella.render.createTextureImageView
import me.hydos.rosella.render.device.Device
import me.hydos.rosella.render.drawToTexture

/**
 * Caches Textures and other texture related objects
 */
class TextureManager(val device: Device) { // TODO: add layers, maybe not in this class but somewhere

	private val textureMap = HashMap<Int, Texture>()
	private val samplerCache = HashMap<SamplerCreateInfo, TextureSampler>() // bro there's like 3 options for this

	private val reusableTexIds = IntPriorityQueues.synchronize(IntArrayPriorityQueue())
	private var nextTexId : Int = 0;

	fun generateTextureId(): Int {
		return if (!reusableTexIds.isEmpty) {
			reusableTexIds.dequeueInt()
		} else {
			nextTexId++
		}
	}

	fun deleteTexture(textureId: Int) {
		textureMap.remove(textureId)
		reusableTexIds.enqueue(textureId)
	}

	fun getTexture(textureId: Int): Texture? {
		return textureMap[textureId];
	}

	fun createTexture(
		engine: Rosella,
		textureId: Int,
		width: Int,
		height: Int,
		imgFormat: Int,
		samplerCreateInfo: SamplerCreateInfo
	) {
		val textureImage = TextureImage(0, 0, 0)
		createTextureImage(engine.renderer, width, height, imgFormat, textureImage)
		textureImage.view = createTextureImageView(device, imgFormat, textureImage.textureImage)

		val textureSampler = samplerCache.computeIfAbsent(samplerCreateInfo) {
			TextureSampler(samplerCreateInfo, engine.device)
		}

		textureMap[textureId] = Texture(imgFormat, width, height, textureImage, textureSampler.pointer);
	}

	fun applySamplerInfoToTexture(
		engine: Rosella,
		textureId: Int,
		samplerCreateInfo: SamplerCreateInfo
	) {
		val textureSampler = samplerCache.computeIfAbsent(samplerCreateInfo) {
			TextureSampler(samplerCreateInfo, engine.device)
		}

		textureMap[textureId]?.textureSampler = textureSampler.pointer
	}

	fun drawToExistingTexture(
		engine: Rosella,
		textureId: Int,
		image: UploadableImage,
		imageRegion: ImageRegion,
	) {
		drawToTexture(engine.device, image, imageRegion, engine.renderer, engine.memory, getTexture(textureId)!!)
	}

	fun drawToExistingTexture(
		engine: Rosella,
		textureId: Int,
		image: UploadableImage
	) {
		drawToTexture(engine.device, image, ImageRegion(0, 0, image.getWidth(), image.getHeight()), engine.renderer, engine.memory, getTexture(textureId)!!)
	}
}