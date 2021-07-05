package me.hydos.rosella.render.material

import me.hydos.rosella.Rosella
import me.hydos.rosella.render.Topology
import me.hydos.rosella.render.resource.Resource
import me.hydos.rosella.render.shader.ShaderProgram
import me.hydos.rosella.render.texture.SamplerCreateInfo
import me.hydos.rosella.render.texture.StbiImage
import me.hydos.rosella.render.texture.Texture
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

	lateinit var texture: Texture

	open fun loadTextures(objectManager: SimpleObjectManager, rosella: Rosella) { //FIXME this is also temporary
		if (resource != Resource.Empty) {
			val test = objectManager.textureManager.generateTextureId() // FIXME this is temporary
			objectManager.textureManager.uploadTextureToId(rosella, test, StbiImage(resource), 0, 0, imgFormat, samplerCreateInfo)
			texture = objectManager.textureManager.getTexture(test)!!
		}
	}
}
