package me.hydos.rosella.render.texture

import me.hydos.rosella.Rosella
import me.hydos.rosella.render.createTextureImage
import me.hydos.rosella.render.createTextureImageView
import me.hydos.rosella.render.createTextureSampler
import me.hydos.rosella.render.device.Device
import me.hydos.rosella.render.resource.Resource

class TextureManager(val device: Device) {

	private val textureMap = HashMap<Resource, Texture>()

	fun getOrLoadTexture(resource: Resource, engine: Rosella, imgFormat: Int): Texture? {
		if (!textureMap.containsKey(resource)) {
			val textureImage = TextureImage(0, 0, 0)
			createTextureImage(device, resource, engine.renderer, engine.memory, imgFormat, textureImage)
			textureImage.view = createTextureImageView(device, imgFormat, textureImage.textureImage)
			val textureSampler = createTextureSampler(device, resource)

			textureMap[resource] = Texture(imgFormat, resource, textureImage, textureSampler)
		}

		return textureMap[resource]
	}
}