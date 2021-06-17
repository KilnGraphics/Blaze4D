package me.hydos.rosella.render.io

import it.unimi.dsi.fastutil.objects.ObjectArrayList
import org.lwjgl.glfw.GLFW.*

/**
 * Represents a window in which Rosella can be attached to
 */
class Window(title: String, width: Int, height: Int, windowResizable: Boolean = true) {
	var monitorWidth: Int = 0
	var monitorHeight: Int = 0

	var fps = 0
	var previousTime = glfwGetTime()
	var frameCount = 0

	val windowPtr: Long
	private val queue: MutableList<() -> JUnit> = ObjectArrayList()
	private val loopCallbacks: MutableList<() -> Unit> = ObjectArrayList()
	private val closeCallbacks: MutableList<() -> Unit> = ObjectArrayList()
	private val resizeCallbacks: MutableList<(width: Int, height: Int) -> Unit> = ObjectArrayList()

	fun startLoop() {
		glfwSetFramebufferSizeCallback(windowPtr, this::onResize)

		while (!glfwWindowShouldClose(windowPtr)) {
			forceMainLoop()
		}
	}

	fun forceMainLoop() {
		glfwPollEvents()
		calculateFps()

		for (callback in loopCallbacks) {
			callback()
		}

		for (function in queue) {
			function.invoke()
		}
		queue.clear()
	}

	private fun calculateFps() {
		val currentTime = glfwGetTime()
		frameCount++
		if (currentTime - previousTime >= 1.0) {
			fps = frameCount
//			println("Fps: $fps")

			frameCount = 0
			previousTime = currentTime
		}
	}

	private fun onResize(window: Long, width: Int, height: Int) {
		for (resizeCallback in resizeCallbacks) {
			resizeCallback(width, height)
		}
	}

	fun onMainLoop(callback: () -> Unit) {
		loopCallbacks.add(callback)
	}

	fun onMainLoop(unit: JUnit) {
		onMainLoop { unit.run() }
	}

	fun onWindowClose(function: () -> Unit) {
		closeCallbacks.add(function)
	}

	fun onWindowResize(function: (width: Int, height: Int) -> Unit) {
		resizeCallbacks.add(function)
	}

	fun queue(unit: JUnit) {
		queue.add { unit }
	}

	init {
		if (!glfwInit()) {
			throw RuntimeException("Cannot Initialize GLFW")
		}
		glfwWindowHint(GLFW_CLIENT_API, GLFW_NO_API)
		glfwWindowHint(GLFW_VISIBLE, GLFW_FALSE)
		glfwWindowHint(GLFW_RESIZABLE, if (windowResizable) GLFW_TRUE else GLFW_FALSE)
		windowPtr = glfwCreateWindow(width, height, title, 0, 0)

		val videoMode = glfwGetVideoMode(glfwGetPrimaryMonitor()) ?: error("Could not start window")
		monitorWidth = videoMode.width()
		monitorHeight = videoMode.height()
	}

	fun close() {
		for (callback in closeCallbacks) {
			callback()
		}
		glfwDestroyWindow(windowPtr)
		glfwTerminate()
	}
}

interface JUnit {
	fun run()
}
