package me.hydos.rosella.example

import me.hydos.rosella.Rosella
import me.hydos.rosella.audio.SoundManager
import me.hydos.rosella.render.io.Window
import me.hydos.rosella.render.material.Material
import me.hydos.rosella.render.model.GuiRenderObject
import me.hydos.rosella.render.resource.Global
import me.hydos.rosella.render.resource.Identifier
import me.hydos.rosella.render.shader.RawShaderProgram
import org.lwjgl.vulkan.VK10

object Portal {

	val screen = Window("Rosella Engine", 1280, 720)
	val engine = Rosella("Portal 3", true, screen)

	val menuBackground = Identifier("example", "menu_background")
	val portalLogo = Identifier("example", "portal_logo")

	val basicShader = Identifier("rosella", "example_shader")
	val guiShader = Identifier("rosella", "gui_shader")

	val background = Identifier("example", "sounds/music/mainmenu/portal2_background01.ogg")

	@JvmStatic
	fun main(args: Array<String>) {
		loadShaders()
		loadMaterials()
		setupMainMenuScene()
		SoundManager.playback(Global.ensureResource(background))
		doMainLoop()
	}

	private fun setupMainMenuScene() {
		engine.addRenderObject(
			GuiRenderObject(
				menuBackground
			).apply {
				scale(1.5f, 1f)
			},
			"mainMenuBackground"
		)

		engine.addRenderObject(
			GuiRenderObject(
				portalLogo,
				-0.9f
			).apply {
				scale(0.4f, 0.1f)
				translate(-1f, -2.6f)
			},
			"portalLogo"
		)
	}

	private fun loadMaterials() {
		engine.registerMaterial(
			menuBackground, Material(
				Global.ensureResource(Identifier("example", "textures/background/background01.png")),
				guiShader,
				VK10.VK_FORMAT_R8G8B8A8_UNORM,
				false
			)
		)
		engine.registerMaterial(
			portalLogo, Material(
				Global.ensureResource(Identifier("example", "textures/gui/portal2logo.png")),
				guiShader,
				VK10.VK_FORMAT_R8G8B8A8_SRGB,
				true
			)
		)
		engine.reloadMaterials()
	}

	private fun loadShaders() {
		engine.registerShader(
			basicShader, RawShaderProgram(
				Global.ensureResource(Identifier("rosella", "shaders/base.v.glsl")),
				Global.ensureResource(Identifier("rosella", "shaders/base.f.glsl")),
				engine.device,
				engine.memory,
				10,
				RawShaderProgram.PoolObjType.UBO,
				RawShaderProgram.PoolObjType.COMBINED_IMG_SAMPLER
			)
		)

		engine.registerShader(
			guiShader, RawShaderProgram(
				Global.ensureResource(Identifier("rosella", "shaders/gui.v.glsl")),
				Global.ensureResource(Identifier("rosella", "shaders/gui.f.glsl")),
				engine.device,
				engine.memory,
				10,
				RawShaderProgram.PoolObjType.UBO,
				RawShaderProgram.PoolObjType.COMBINED_IMG_SAMPLER
			)
		)
	}

	private fun doMainLoop() {
		engine.renderer.rebuildCommandBuffers(engine.renderer.renderPass, engine)
		screen.onMainLoop {
			engine.renderer.render(engine)
		}
		screen.start()
	}
}
