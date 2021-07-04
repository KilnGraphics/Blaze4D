package me.hydos.rosella.render.texture

import it.unimi.dsi.fastutil.ints.IntArrayPriorityQueue
import it.unimi.dsi.fastutil.ints.IntPriorityQueues
import me.hydos.rosella.render.createTextureImage
import me.hydos.rosella.render.createTextureImageView
import me.hydos.rosella.vkobjects.VkCommon

/**
 * Caches Textures and other texture related objects
 */
class TextureManager(val common: VkCommon) { // TODO: add layers, maybe not in this class but somewhere

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

	// TODO: add variant of this method which accepts a pointer or a buffer directly
	// probably wont work if we need to mess with the buffer for the capacity and location and stuff for vulkan
	// for nativeimage, we could also change the format so we don't have to change the channels
	fun uploadTextureToId(
		engine: Rosella,
		textureId: Int,
		image: UploadableImage,
		offsetX: Int,
		offsetY: Int,
		imgFormat: Int,
		samplerCreateInfo: SamplerCreateInfo
	) {
		val textureImage = TextureImage(0, 0, 0)
		createTextureImage(device, image, offsetX, offsetY, engine.renderer, engine.memory, imgFormat, textureImage)
		textureImage.view = createTextureImageView(device, imgFormat, textureImage.textureImage)

		val textureSampler = samplerCache.computeIfAbsent(samplerCreateInfo) {
			TextureSampler(samplerCreateInfo, engine.device)
		}

		textureMap[textureId] = Texture(imgFormat, textureImage, textureSampler.pointer);
	}
}