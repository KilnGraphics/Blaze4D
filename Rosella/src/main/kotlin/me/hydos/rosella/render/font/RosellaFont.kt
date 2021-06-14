package me.hydos.rosella.render.font

import me.hydos.rosella.Rosella
import me.hydos.rosella.render.Topology
import me.hydos.rosella.render.material.Material
import me.hydos.rosella.render.model.RenderObject
import me.hydos.rosella.render.model.ShapeRenderObject
import me.hydos.rosella.render.resource.Global
import me.hydos.rosella.render.resource.Identifier
import me.hydos.rosella.render.shader.RawShaderProgram
import org.joml.Vector3f
import org.lwjgl.vulkan.VK10
import java.awt.Font
import java.awt.Graphics2D
import java.awt.image.BufferedImage

class RosellaFont(private val font: Font, rosella: Rosella) {

	private val fontShader = Identifier("rosella", "font_shader")
	private val fontMaterial = Identifier("rosella", "font_texture")

	init {
		rosella.registerShader(
			fontShader, RawShaderProgram(
				Global.ensureResource(Identifier("rosella", "shaders/fonts.v.glsl")),
				Global.ensureResource(Identifier("rosella", "shaders/fonts.f.glsl")),
				rosella.device,
				rosella.memory,
				99999,
				RawShaderProgram.PoolObjType.UBO,
				RawShaderProgram.PoolObjType.COMBINED_IMG_SAMPLER
			)
		)

		rosella.registerMaterial(
			fontMaterial, Material(
				Global.fromBufferedImage(BufferedImage(1, 1, BufferedImage.TYPE_3BYTE_BGR), fontMaterial),
				fontShader,
				VK10.VK_FORMAT_R8G8B8A8_UNORM,
				false,
				Topology.TRIANGLES
			)
		)
	}

	fun createString(
		string: String,
		colour: Vector3f,
		z: Float,
		scale: Float,
		translateX: Float,
		translateZ: Float
	): RenderObject {
		val graphics = BufferedImage(1920, 1080, BufferedImage.TYPE_4BYTE_ABGR).graphics as Graphics2D
		val outline = font.createGlyphVector(graphics.fontRenderContext, string).outline
		return ShapeRenderObject(
			outline,
			fontMaterial,
			z,
			colour,
			scale,
			scale,
			translateX,
			translateZ
		)
	}
}
