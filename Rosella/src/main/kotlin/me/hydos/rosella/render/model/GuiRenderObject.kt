package me.hydos.rosella.render.model

import me.hydos.rosella.render.`object`.RenderObject
import me.hydos.rosella.render.material.Material
import me.hydos.rosella.render.resource.Resource
import me.hydos.rosella.render.util.memory.Memory
import me.hydos.rosella.vkobjects.VkCommon
import org.joml.Vector3f

open class GuiRenderObject(
	material: Material,
	var z: Float = -1f,
	var colour: Vector3f = Vector3f(0f, 0f, 0f)
) : RenderObject(Resource.Empty, material) {

	constructor(material: Material, z: Float, colour: Vector3f, scaleX: Float, scaleZ: Float) : this(material, z, colour) {
		scale(scaleX, scaleZ)
	}

	constructor(
		material: Material,
		z: Float,
		colour: Vector3f,
		scaleX: Float,
		scaleZ: Float,
		translateX: Float,
		translateZ: Float
	) : this(material, z, colour, scaleX, scaleZ) {
		translate(translateX, translateZ)
	}

	override fun loadModelInfo() {
		renderInfo.consumer.clear()
		renderInfo.indices = ArrayList()

		colour = Vector3f(0f, 0f, 0f)

		renderInfo.consumer
			.pos(-0.5f, -0.5f, 0f)
			.color(colour.x().toInt(), colour.y().toInt(), colour.z().toInt())
			.uv(0f, 0f)
			.nextVertex()

		renderInfo.consumer
			.pos(0.5f, -0.5f, 0f)
			.color(colour.x().toInt(), colour.y().toInt(), colour.z().toInt())
			.uv(1f, 0f)
			.nextVertex()

		renderInfo.consumer
			.pos(0.5f, 0.5f, 0f)
			.color(colour.x().toInt(), colour.y().toInt(), colour.z().toInt())
			.uv(1f, 1f)
			.nextVertex()

		renderInfo.consumer
			.pos(-0.5f, 0.5f, 0f)
			.color(colour.x().toInt(), colour.y().toInt(), colour.z().toInt())
			.uv(0f, 1f)
			.nextVertex()

		renderInfo.indices.add(0)
		renderInfo.indices.add(1)
		renderInfo.indices.add(2)
		renderInfo.indices.add(2)
		renderInfo.indices.add(3)
		renderInfo.indices.add(0)
	}

	override fun onAddedToScene(common: VkCommon, memory: Memory) {
		super.onAddedToScene(common, memory)
		modelMatrix.translate(0f, 0f, z)
	}

	fun scale(x: Float, y: Float) {
		modelMatrix.scale(x, y, 1f)
	}

	fun translate(x: Float, y: Float) {
		modelMatrix.translate(x, -y, 0f)
	}
}