package me.hydos.rosella.render.texture

data class Texture(
    val imgFormat: Int,
    val width: Int,
    val height: Int,
    val textureImage: TextureImage,
    var textureSampler: Long?
)
