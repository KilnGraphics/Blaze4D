package me.hydos.rosella.scene

import me.hydos.rosella.render.resource.Identifier

class Scene {
	var id: Identifier = Identifier("rosella", "empty")
	var models: MutableList<SceneModel> = ArrayList()
}