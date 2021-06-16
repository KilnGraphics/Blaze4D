package me.hydos.rosella.render.model

import me.hydos.rosella.Rosella
import me.hydos.rosella.render.resource.Identifier
import me.hydos.rosella.render.resource.Resource
import me.hydos.rosella.render.shader.ubo.LowLevelUbo
import org.joml.Vector2f
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
		vertices = ArrayList()
		indices = ArrayList()

		vertices.add(Vertex(Vector3f(-0.5f, -0.5f, 0f), colour, Vector2f(0f, 0f)))
		vertices.add(Vertex(Vector3f(0.5f, -0.5f, 0f), colour, Vector2f(1f, 0f)))
		vertices.add(Vertex(Vector3f(0.5f, 0.5f, 0f), colour, Vector2f(1f, 1f)))
		vertices.add(Vertex(Vector3f(-0.5f, 0.5f, 0f), colour, Vector2f(0f, 1f)))

		indices.add(0)
		indices.add(1)
		indices.add(2)
		indices.add(2)
		indices.add(3)
		indices.add(0)
	}

	override fun load(engine: Rosella) {
		val retrievedMaterial = engine.materials[materialIdentifier]
			?: error("The material $materialIdentifier couldn't be found. (Are you registering the material?)")
		mat = retrievedMaterial
		uniformBufferObject = LowLevelUbo(engine.device, engine.memory)
		uniformBufferObject.create(engine.renderer.swapChain)
		modelTransformMatrix.translate(0f, 0f, z)
	}

	fun scale(x: Float, y: Float) {
		modelTransformMatrix.scale(x, y, 1f)
	}

	fun translate(x: Float, y: Float) {
		modelTransformMatrix.translate(x, -y, 0f)
	}
}