package me.hydos.rosella.render.texture

import me.hydos.rosella.Rosella
import me.hydos.rosella.render.createTextureImage
import me.hydos.rosella.render.createTextureImageView
import me.hydos.rosella.render.createTextureSampler
import me.hydos.rosella.render.device.Device
import me.hydos.rosella.render.resource.Resource

class TextureManager(val device: Device) {

	private val resourceTextureMap = HashMap<Resource, Texture>()
	private val textureMap = HashMap<UploadableImage, Texture>()

	@Deprecated("Resource Specification is bad")
	fun getOrLoadTexture(resource: Resource, engine: Rosella, imgFormat: Int): Texture? {
		if (!resourceTextureMap.containsKey(resource)) {
			val textureImage = TextureImage(0, 0, 0)
			createTextureImage(device, StbiImage(resource), engine.renderer, engine.memory, imgFormat, textureImage)
			textureImage.view = createTextureImageView(device, imgFormat, textureImage.textureImage)
			val textureSampler = createTextureSampler(device)

			resourceTextureMap[resource] = Texture(imgFormat, textureImage, textureSampler)
		}

		return resourceTextureMap[resource]
	}

	fun getOrLoadTexture(image: UploadableImage, engine: Rosella, imgFormat: Int): Texture? {
		if (!textureMap.containsKey(image)) {
			val textureImage = TextureImage(0, 0, 0)
			createTextureImage(device, image, engine.renderer, engine.memory, imgFormat, textureImage)
			textureImage.view = createTextureImageView(device, imgFormat, textureImage.textureImage)
			val textureSampler = createTextureSampler(device)

			textureMap[image] = Texture(imgFormat, textureImage, textureSampler)
		}

		return textureMap[image]
	}
}