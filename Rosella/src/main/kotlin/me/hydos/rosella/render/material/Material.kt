package me.hydos.rosella.render.material

import me.hydos.rosella.Rosella
import me.hydos.rosella.render.Topology
import me.hydos.rosella.render.resource.Resource
import me.hydos.rosella.render.shader.ShaderProgram
import me.hydos.rosella.render.texture.*
import me.hydos.rosella.render.vertex.VertexFormat
import me.hydos.rosella.scene.`object`.impl.SimpleObjectManager

/**
 * A Material is like texture information, normal information, and all of those things which give an object character wrapped into one class.
 * similar to how unity material's works
 * guaranteed to change in the future
 */
open class Material(
	val resource: Resource,
	var shader: ShaderProgram?,
	val imgFormat: Int,
	val useBlend: Boolean,
	val topology: Topology,
	val vertexFormat: VertexFormat,
	val samplerCreateInfo: SamplerCreateInfo
) {
	lateinit var pipeline: PipelineInfo

	lateinit var textures: Array<Texture?>

	open fun loadTextures(objectManager: SimpleObjectManager, rosella: Rosella) { //FIXME this is also temporary
		if (resource != Resource.Empty) {
			val textureManager = objectManager.textureManager
			val textureId = textureManager.generateTextureId() // FIXME this texture can't be removed
			val image: UploadableImage = StbiImage(resource)
			textureManager.createTexture(
				rosella.renderer,
				textureId,
				image.getWidth(),
				image.getHeight(),
				imgFormat
			)
			textureManager.setTextureSampler(textureId, 0, samplerCreateInfo) // 0 is the default texture no, but it's still gross
			textureManager.drawToExistingTexture(rosella.renderer, rosella.memory, textureId, image)
			val texture = textureManager.getTexture(textureId)!!
			textures = arrayOf(texture) //FIXME THIS SUCKS
		}
	}
}
