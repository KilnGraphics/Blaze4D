package me.hydos.rosella.render.device

import java.util.stream.IntStream

class QueueFamilyIndices {
	internal var graphicsFamily: Int? = null
	internal var presentFamily: Int? = null

	val isComplete: Boolean
		get() = graphicsFamily != null && presentFamily != null

	fun unique(): IntArray {
		return IntStream.of(graphicsFamily!!, presentFamily!!).distinct().toArray()
	}

	fun array(): IntArray {
		return intArrayOf(graphicsFamily!!, presentFamily!!)
	}
}
