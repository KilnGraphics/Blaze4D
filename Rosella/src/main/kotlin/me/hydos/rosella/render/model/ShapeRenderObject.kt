package me.hydos.rosella.render.model

import me.hydos.rosella.render.resource.Identifier
import org.joml.Vector3f
import java.awt.Shape
import java.awt.geom.AffineTransform
import java.awt.geom.PathIterator
import java.util.function.Function
import kotlin.math.max

class ShapeRenderObject(
	private val shape: Shape,
	matId: Identifier,
	z: Float,
	colour: Vector3f,
	scaleX: Float,
	scaleZ: Float,
	translateX: Float,
	translateZ: Float
) : GuiRenderObject(matId, z, colour, scaleX, scaleZ, translateX, translateZ) {

	private val steps = 100

	override fun loadModelInfo() {
		val iterator = shape.getPathIterator(AffineTransform().apply {
			val i = shape.bounds2D.run {
				max(width - x, height - y).toFloat()
			}

			setToScale(10.0 / i, -10.0 / i)
		})
		val buffer = FloatArray(6)

		var location = 0f to 0f

		while (!iterator.isDone) {
			when (iterator.currentSegment(buffer)) {
				PathIterator.SEG_MOVETO -> {
					location = buffer[0] to buffer[1]
				}
				PathIterator.SEG_LINETO -> {
					val point = buffer[0] to buffer[1]
					drawLine(location, point)
					location = point
				}
				PathIterator.SEG_QUADTO -> {
					val a = buffer[0] to buffer[1]
					val b = buffer[2] to buffer[3]
					val f = run {
						val function = interpolate(
							location.first,
							location.second,
							a.first,
							a.second,
							b.first,
							b.second
						)

						({ a: Int ->
							val x = a / steps.toFloat()
							val y = function.apply(x)
							x to y
						})
					}

					var last = f(0)

					for (x in 0..steps) {
						val new = f(x)
						drawLine(last, new)
						last = new
					}
				}
				PathIterator.SEG_CUBICTO -> {
					val a = buffer[0] to buffer[1]
					val b = buffer[2] to buffer[3]
					val c = buffer[4] to buffer[5]
					val f = run {
						val function = interpolate(
							location.first,
							location.second,
							a.first,
							a.second,
							b.first,
							b.second,
							c.first,
							c.second
						)

						({ a: Int ->
							val x = a / steps.toFloat()
							val y = function.apply(x)
							x to y
						})
					}

					var last = f(0)

					for (x in 0..steps) {
						val new = f(x)
						drawLine(last, new)
						last = new
					}
				}
				PathIterator.SEG_CLOSE -> {
				}
				else -> {
					error("Unknown path segment type")
				}
			}

			iterator.next()
		}
	}

	private fun drawLine(from: Pair<Float, Float>, to: Pair<Float, Float>) {
		renderInfo.consumer
			.pos(from.first + 0.02f, from.second + 0.02f, 0f)
			.color(colour.x().toInt(), colour.y().toInt(), colour.z().toInt())
			.uv(0f, 0f)
			.nextVertex()

		renderInfo.consumer
			.pos(from.first, from.second, 0f)
			.color(colour.x().toInt(), colour.y().toInt(), colour.z().toInt())
			.uv(0f, 0f)
			.nextVertex()

		renderInfo.consumer
			.pos(to.first, to.second, 0f)
			.color(colour.x().toInt(), colour.y().toInt(), colour.z().toInt())
			.uv(0f, 0f)
			.nextVertex()

		renderInfo.indices.add(renderInfo.consumer.getVertexCount() - 3)
		renderInfo.indices.add(renderInfo.consumer.getVertexCount() - 2)
		renderInfo.indices.add(renderInfo.consumer.getVertexCount() - 1)
	}

	fun interpolate(x1: Float, y1: Float, x2: Float, y2: Float, x3: Float, y3: Float): Function<Float, Float> {
		val l0d = (x1 - x2) * (x1 - x3)
		val l1d = (x2 - x1) * (x2 - x3)
		val l2d = (x3 - x1) * (x3 - x2)
		return Function { x: Float ->
			val l_0 = (x - x2) * (x - x3)
			val l_1 = (x - x1) * (x - x3)
			val l_2 = (x - x1) * (x - x2)
			y1 * l_0 / l0d + y2 * l_1 / l1d + y3 * l_2 / l2d
		}
	}

	fun interpolate(
		x1: Float,
		y1: Float,
		x2: Float,
		y2: Float,
		x3: Float,
		y3: Float,
		x4: Float,
		y4: Float
	): Function<Float, Float> {
		val l0d = (x1 - x2) * (x1 - x3) * (x1 - x4)
		val l1d = (x2 - x1) * (x2 - x3) * (x2 - x4)
		val l2d = (x3 - x1) * (x3 - x2) * (x3 - x4)
		val l3d = (x4 - x1) * (x4 - x2) * (x4 - x3)
		return Function { x: Float ->
			val l_0 = (x - x2) * (x - x3) * (x - x4)
			val l_1 = (x - x1) * (x - x3) * (x - x4)
			val l_2 = (x - x1) * (x - x2) * (x - x4)
			val l_3 = (x - x1) * (x - x2) * (x - x3)
			y1 * l_0 / l0d + y2 * l_1 / l1d + y3 * l_2 / l2d + y4 * l_3 / l3d
		}
	}

	private fun test(x1: Float, y1: Float, x2: Float, y2: Float, x3: Float, y3: Float) {
		val function = interpolate(x1, y1, x2, y2, x3, y3)
		if (function.apply(x1) != y1) {
			throw AssertionError()
		}
		if (function.apply(x2) != y2) {
			throw AssertionError()
		}
		if (function.apply(x3) != y3) {
			throw AssertionError()
		}
	}

	private fun test(x1: Float, y1: Float, x2: Float, y2: Float, x3: Float, y3: Float, x4: Float, y4: Float) {
		val function = interpolate(x1, y1, x2, y2, x3, y3, x4, y4)
		if (function.apply(x1) != y1) {
			throw AssertionError()
		}
		if (function.apply(x2) != y2) {
			throw AssertionError()
		}
		if (function.apply(x3) != y3) {
			throw AssertionError()
		}
		if (function.apply(x4) != y4) {
			throw AssertionError()
		}
	}
}
