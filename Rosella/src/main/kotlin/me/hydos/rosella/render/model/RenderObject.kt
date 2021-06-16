package me.hydos.rosella.render.model

import me.hydos.rosella.Rosella
import me.hydos.rosella.render.material.Material
import me.hydos.rosella.render.renderer.Renderer
import me.hydos.rosella.render.resource.Identifier
import me.hydos.rosella.render.resource.Resource
import me.hydos.rosella.render.shader.ubo.LowLevelUbo
import me.hydos.rosella.render.shader.ubo.Ubo
import me.hydos.rosella.render.util.memory.Memory
import org.joml.Matrix4f
import org.joml.Vector3f
import org.joml.Vector3fc
import org.lwjgl.assimp.Assimp
import org.lwjgl.util.vma.Vma.vmaFreeMemory

open class RenderObject(private val model: Resource, val materialIdentifier: Identifier) : Renderable {

	var vertices: ArrayList<Vertex> = ArrayList()
	var indices: ArrayList<Int> = ArrayList()
	private var vertexBuffer: Long = 0
	private var indexBuffer: Long = 0

	private var descSets: MutableList<Long> = ArrayList()
	var modelTransformMatrix: Matrix4f = Matrix4f()
	lateinit var uniformBufferObject: Ubo
	lateinit var mat: Material

	override fun load(engine: Rosella) {
		val retrievedMaterial = engine.materials[materialIdentifier]
			?: error("The material $materialIdentifier couldn't be found. (Are you registering the material?)")
		mat = retrievedMaterial
		uniformBufferObject = LowLevelUbo(engine.device, engine.memory)
		uniformBufferObject.create(engine.renderer.swapChain)
	}

	override fun free(memory: Memory) {
		vmaFreeMemory(memory.allocator, vertexBuffer)
		vmaFreeMemory(memory.allocator, indexBuffer)
		uniformBufferObject.free()
	}

	override fun create(engine: Rosella) {
		loadModelInfo()
		vertexBuffer = engine.memory.createVertexBuffer(engine, vertices)
		indexBuffer = engine.memory.createIndexBuffer(engine, indices)
		resize(engine.renderer)
	}

	override fun resize(renderer: Renderer) {
		mat.shader.raw.createDescriptorSets(renderer.swapChain, this)
	}

	override fun getIndices(): List<Int> {
		return indices
	}

	override fun getVertices(): List<Vertex> {
		return vertices
	}

	override fun getDescriptorSets(): MutableList<Long> {
		return descSets
	}

	override fun setDescriptorSets(descSets: MutableList<Long>) {
		this.descSets = descSets
	}

	override fun getMaterial(): Material {
		return mat
	}

	override fun getVerticesBuffer(): Long {
		return vertexBuffer
	}

	override fun getIndicesBuffer(): Long {
		return indexBuffer
	}

	override fun getUbo(): Ubo {
		return uniformBufferObject
	}

	override fun getTransformMatrix(): Matrix4f {
		return modelTransformMatrix
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