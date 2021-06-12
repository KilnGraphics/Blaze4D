package me.hydos.rosella.render.texture

import me.hydos.rosella.render.resource.Resource

data class Texture(
	val imgFormat: Int,
	val resource: Resource,
	val textureImage: TextureImage,
	val textureSampler: Long
)
