package me.hydos.rosella.render.texture

import me.hydos.rosella.Rosella
import me.hydos.rosella.render.createTextureImage
import me.hydos.rosella.render.createTextureImageView
import me.hydos.rosella.render.device.Device

/**
 * Caches Textures and other texture related objects
 */
class TextureManager(val device: Device) {

	private val textureCache = HashMap<UploadableImage, Texture>()
	private val samplerCache = HashMap<SamplerCreateInfo, TextureSampler>()

	fun getOrLoadTexture(
		image: UploadableImage,
		engine: Rosella,
		imgFormat: Int,
		samplerCreateInfo: SamplerCreateInfo
	): Texture? {
		textureCache.computeIfAbsent(image) {
			val textureImage = TextureImage(0, 0, 0)
			createTextureImage(device, image, engine.renderer, engine.memory, imgFormat, textureImage)
			textureImage.view = createTextureImageView(device, imgFormat, textureImage.textureImage)

			val textureSampler = samplerCache.computeIfAbsent(samplerCreateInfo) {
				TextureSampler(samplerCreateInfo, engine.device)
			}

			Texture(imgFormat, textureImage, textureSampler.pointer)
		}
		return textureCache[image]
	}
}