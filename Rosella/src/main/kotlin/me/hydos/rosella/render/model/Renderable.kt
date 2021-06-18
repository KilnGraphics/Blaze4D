package me.hydos.rosella.render.model

import me.hydos.rosella.Rosella
import me.hydos.rosella.render.material.Material
import me.hydos.rosella.render.renderer.Renderer
import me.hydos.rosella.render.shader.ubo.Ubo
import me.hydos.rosella.render.util.memory.Memory
import me.hydos.rosella.render.vertex.VertexConsumer
import org.joml.Matrix4f

interface Renderable {
	fun load(engine: Rosella)
	fun free(memory: Memory)
	fun create(engine: Rosella)
	fun resize(renderer: Renderer)
	fun getIndices(): List<Int>
	fun render(): VertexConsumer
	fun getDescriptorSets(): MutableList<Long>
	fun setDescriptorSets(descSets: MutableList<Long>)
	fun getMaterial(): Material
	fun getVerticesBuffer(): Long
	fun getIndicesBuffer(): Long
	fun getUbo(): Ubo
	fun getTransformMatrix(): Matrix4f
}