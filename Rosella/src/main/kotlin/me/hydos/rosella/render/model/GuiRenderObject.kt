package me.hydos.rosella.render.model

import me.hydos.rosella.Rosella
import me.hydos.rosella.render.`object`.RenderObject
import me.hydos.rosella.render.resource.Identifier
import me.hydos.rosella.render.resource.Resource
import org.joml.Vector3f

open class GuiRenderObject(
	materialIdentifier: Identifier,
	var z: Float = -1f,
	var colour: Vector3f = Vector3f(0f, 0f, 0f)
) : RenderObject(Resource.Empty, materialIdentifier) {

	constructor(matId: Identifier, z: Float, colour: Vector3f, scaleX: Float, scaleZ: Float) : this(matId, z, colour) {
		scale(scaleX, scaleZ)
	}

	constructor(
		matId: Identifier,
		z: Float,
		colour: Vector3f,
		scaleX: Float,
		scaleZ: Float,
		translateX: Float,
		translateZ: Float
	) : this(matId, z, colour, scaleX, scaleZ) {
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

	override fun onAddedToScene(engine: Rosella) {
		super.onAddedToScene(engine)
		modelMatrix.translate(0f, 0f, z)
	}

	fun scale(x: Float, y: Float) {
		modelMatrix.scale(x, y, 1f)
	}

	fun translate(x: Float, y: Float) {
		modelMatrix.translate(x, -y, 0f)
	}
}