package me.hydos.rosella.editor

import me.hydos.rosella.Rosella
import me.hydos.rosella.gui.Canvas
import me.hydos.rosella.gui.Layer
import me.hydos.rosella.render.io.Window
import me.hydos.rosella.render.resource.Identifier
import org.joml.Vector3f
import org.lwjgl.vulkan.VK10

object Editor {

	private val window: Window = Window("Rosella Scene Editor", 1920, 1080, true)
	private val rosella: Rosella = Rosella("sceneEditor", false, window)

	private val folder = Identifier("rosella", "folder")

	@JvmStatic
	fun main(args: Array<String>) {
		createGui()
		rosella.renderer.rebuildCommandBuffers(rosella.renderer.renderPass, rosella)
		window.onMainLoop {
			rosella.renderer.render(rosella)
		}
		window.start()
	}

	private fun createGui() {
		val canvas = Canvas(rosella, window)

		rosella.registerMaterial(
			folder, canvas.createGuiMaterial(
				Identifier("rosella", "editor/gui/folder.png"),
				VK10.VK_FORMAT_R8G8B8A8_SRGB,
				true
			)
		)
		rosella.reloadMaterials()

		canvas.addRect("testSquare", 0, 0, 100, 100, Layer.FOREGROUND1, Vector3f(46 / 255f, 209 / 255f, 84 / 255f))
	}
}