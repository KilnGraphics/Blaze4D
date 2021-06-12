package me.hydos.rosella.render.model

import me.hydos.rosella.Rosella
import me.hydos.rosella.render.material.Material
import me.hydos.rosella.render.renderer.Renderer
import me.hydos.rosella.render.resource.Identifier
import me.hydos.rosella.render.resource.Resource
import me.hydos.rosella.render.shader.ubo.BasicUbo
import me.hydos.rosella.render.shader.ubo.Ubo
import me.hydos.rosella.render.util.memory.Memory
import org.joml.Matrix4f
import org.joml.Vector3f
import org.joml.Vector3fc
import org.lwjgl.assimp.Assimp
import org.lwjgl.util.vma.Vma.vmaFreeMemory


open class RenderObject(private val model: Resource, val materialIdentifier: Identifier) {

	var vertices: ArrayList<Vertex> = ArrayList()
	var indices: ArrayList<Int> = ArrayList()

	var vertexBuffer: Long = 0
	var indexBuffer: Long = 0

	var descriptorSets: MutableList<Long> = ArrayList()

	var modelTransformMatrix: Matrix4f = Matrix4f()

	lateinit var ubo: Ubo

	lateinit var material: Material

	open fun load(engine: Rosella) {
		val retrievedMaterial = engine.materials[materialIdentifier]
			?: error("The material $materialIdentifier couldn't be found. (Are you registering the material?)")
		material = retrievedMaterial
		ubo = BasicUbo(engine.device, engine.memory)
		ubo.create(engine.renderer.swapChain)
	}

	fun free(memory: Memory) {
		vmaFreeMemory(memory.allocator, vertexBuffer)
		vmaFreeMemory(memory.allocator, indexBuffer)
		ubo.free()
	}

	fun create(engine: Rosella) {
		loadModelInfo()
		vertexBuffer = engine.memory.createVertexBuffer(engine, vertices)
		indexBuffer = engine.memory.createIndexBuffer(engine, indices)
		material.shader.raw.createDescriptorSets(engine.renderer.swapChain, this)
	}

	fun resize(renderer: Renderer) {
		material.shader.raw.createDescriptorSets(renderer.swapChain, this)
	}

	open fun loadModelInfo() {
		val model: ModelLoader.SimpleModel =
			ModelLoader.loadModel(model, Assimp.aiProcess_FlipUVs or Assimp.aiProcess_DropNormals)
		val vertexCount: Int = model.positions.size

		vertices = ArrayList()

		val color: Vector3fc = Vector3f(1.0f, 1.0f, 1.0f)
		for (i in 0 until vertexCount) {
			vertices.add(
				Vertex(
					model.positions[i],
					color,
					model.texCoords[i]
				)
			)
		}

		indices = ArrayList(model.indices.size)

		for (i in 0 until model.indices.size) {
			indices.add(model.indices[i])
		}
	}
}