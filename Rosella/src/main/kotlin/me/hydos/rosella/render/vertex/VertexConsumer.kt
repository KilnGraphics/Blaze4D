package me.hydos.rosella.render.vertex

/**
 * A consumer of vertices. puts all vertex info into a buffer for copying to memory
 */
interface VertexConsumer {

	val format: VertexFormat

	fun pos(x: Float, y: Float, z: Float): VertexConsumer

	fun color(red: Byte, green: Byte, blue: Byte): VertexConsumer

	fun color(red: Byte, green: Byte, blue: Byte, alpha: Byte): VertexConsumer

	fun uv(u: Float, v: Float): VertexConsumer

	fun uv(u: Short, v: Short): VertexConsumer

	fun light(u: Short, v: Short): VertexConsumer {
		this.uv(u, v)
		return this
	}

	fun normal(x :Float, y: Float, z: Float): VertexConsumer

	fun nextVertex(): VertexConsumer

	fun clear()

	fun getVertexSize(): Int

	fun getVertexCount(): Int

	fun copy(): VertexConsumer
}