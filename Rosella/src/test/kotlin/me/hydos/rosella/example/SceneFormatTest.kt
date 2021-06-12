package me.hydos.rosella.example

import com.google.gson.Gson
import me.hydos.rosella.render.resource.Identifier
import me.hydos.rosella.scene.Scene
import me.hydos.rosella.scene.SceneModel

object SceneFormatTest {

	val GSON: Gson = Gson()

	@JvmStatic
	fun main(args: Array<String>) {
		val scene = Scene()
		val sceneObj = SceneModel().apply {
			id = Identifier("example", "aCoolObject")
			location = Identifier("rosella", "empty")
		}

		scene.models.add(sceneObj)

		println(GSON.toJson(scene))
	}
}