package me.hydos.rosella.render.camera

import me.hydos.rosella.render.io.Window
import me.hydos.rosella.render.swapchain.SwapChain
import org.joml.Matrix4f
import org.lwjgl.glfw.GLFW.*
import org.lwjgl.glfw.GLFWKeyCallback

class Camera(window: Window) {
	var view: Matrix4f = Matrix4f()
	var proj: Matrix4f = Matrix4f()

	init {
		glfwSetKeyCallback(window.windowPtr, object : GLFWKeyCallback() {
			override fun invoke(window: Long, key: Int, scancode: Int, action: Int, mods: Int) {
				if (key == GLFW_KEY_W) {
					view.translate(0f, -4f, 0f)
				}
				if (key == GLFW_KEY_S) {
					view.translate(0f, 4f, 0f)
				}
			}
		})
	}

	fun createViewAndProj(swapChain: SwapChain) {
		view = Matrix4f()
		proj = Matrix4f()

		view.lookAt(2.0f, -40.0f, 2.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 1.0f)
		proj.perspective(
			Math.toRadians(45.0).toFloat(),
			swapChain.swapChainExtent.width().toFloat() / swapChain.swapChainExtent.height().toFloat(),
			0.1f,
			1000.0f
		)
		proj.m11(proj.m11() * -1)
	}
}