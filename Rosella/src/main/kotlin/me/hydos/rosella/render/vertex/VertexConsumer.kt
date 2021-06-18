package me.hydos.rosella.render.vertex

/**
 * A consumer of vertices. puts all vertex info into a buffer for copying to memory
 */
interface VertexConsumer {

	fun pos(x: Float, y: Float, z: Float): VertexConsumer

	fun color(red: Int, green: Int, blue: Int): VertexConsumer

	fun uv(u: Float, v: Float): VertexConsumer

	fun nextVertex(): VertexConsumer

	fun clear()

	fun getVertexSize(): Int

	fun getVertexCount(): Int
}