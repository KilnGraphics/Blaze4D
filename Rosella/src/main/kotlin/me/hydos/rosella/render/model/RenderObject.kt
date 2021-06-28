package me.hydos.rosella.render.model

import me.hydos.rosella.Rosella
import me.hydos.rosella.render.device.Device
import me.hydos.rosella.render.material.Material
import me.hydos.rosella.render.resource.Identifier
import me.hydos.rosella.render.resource.Resource
import me.hydos.rosella.render.shader.ubo.LowLevelUbo
import me.hydos.rosella.render.shader.ubo.Ubo
import me.hydos.rosella.render.util.memory.BufferInfo
import me.hydos.rosella.render.util.memory.Memory
import me.hydos.rosella.render.vertex.BufferVertexConsumer
import me.hydos.rosella.render.vertex.VertexConsumer
import me.hydos.rosella.render.vertex.VertexFormats
import org.joml.Matrix4f
import org.joml.Vector3f
import org.joml.Vector3fc
import org.lwjgl.assimp.Assimp

open class RenderObject(private val model: Resource, val materialIdentifier: Identifier) : Renderable {

	var consumer = BufferVertexConsumer(VertexFormats.POSITION_COLOR_UV)
	var indices: ArrayList<Int> = ArrayList()
	private lateinit var vertexBuffer: BufferInfo
	private lateinit var indexBuffer: BufferInfo

	private var descSets: MutableList<Long> = ArrayList()
	var modelTransformMatrix: Matrix4f = Matrix4f()
	lateinit var uniformBufferObject: Ubo
	lateinit var mat: Material

	override fun load(engine: Rosella) {
		val retrievedMaterial = engine.materials[materialIdentifier]
			?: error("The material $materialIdentifier couldn't be found. (Are you registering the material?)")
		mat = retrievedMaterial
		uniformBufferObject = LowLevelUbo(engine.device, engine.memory)
		uniformBufferObject.create(engine.renderer.swapchain)
	}

	override fun free(memory: Memory, device: Device) {
		memory.freeBuffer(vertexBuffer)
		memory.freeBuffer(indexBuffer)
		uniformBufferObject.free()
	}

	override fun create(engine: Rosella) {
		loadModelInfo()
		vertexBuffer = engine.memory.createVertexBuffer(engine, consumer)
		indexBuffer = engine.memory.createIndexBuffer(engine, indices)
		resize(engine)
	}

	override fun resize(engine: Rosella) {
		mat.shader.raw.createDescriptorSets(engine, this)
	}

	override fun getIndices(): List<Int> {
		return indices
	}

	override fun render(): VertexConsumer {
		return consumer
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

	override fun getVerticesBuffer(): BufferInfo {
		return vertexBuffer
	}

	override fun getIndicesBuffer(): BufferInfo {
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

		consumer.clear()
		val color: Vector3fc = Vector3f(1.0f, 1.0f, 1.0f)
		for (i in 0 until vertexCount) {
			val pos = model.positions[i]
			val uvs = model.texCoords[i]
			consumer
				.pos(pos.x(), pos.y(), pos.z())
				.color(color.x().toInt(), color.y().toInt(), color.z().toInt())
				.uv(uvs.x(), uvs.y())
				.nextVertex()
		}

		indices = ArrayList(model.indices.size)

		for (i in 0 until model.indices.size) {
			indices.add(model.indices[i])
		}
	}
}