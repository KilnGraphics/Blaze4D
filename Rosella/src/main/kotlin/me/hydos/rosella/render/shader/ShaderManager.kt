package me.hydos.rosella.render.shader

import me.hydos.rosella.render.resource.Identifier
import me.hydos.rosella.vkobjects.VkCommon

class ShaderManager(val common: VkCommon) {

	var shaders = HashMap<Identifier, RawShaderProgram>()
	var cachedShaders = HashMap<Identifier, ShaderProgram>()

	fun getOrCreateShader(identifier: Identifier) : ShaderProgram? {
		if(!cachedShaders.containsKey(identifier)) {
			val rawShader = shaders[identifier]!!
			cachedShaders[identifier] = ShaderProgram(rawShader, rosella, rawShader.maxObjCount)
		}

		return cachedShaders[identifier]
	}

//	@Deprecated("Try not to store instances of a raw shader program. only store an Identifier or store an ShaderProgram")
	fun getOrCreateShader(rawShader: RawShaderProgram): ShaderProgram? {
		for (identifier in shaders.keys) {
			if(rawShader == shaders[identifier]) {
				return getOrCreateShader(identifier)
			}
		}
		error("Couldn't find a loaded shader matching that. oh no")
	}

	fun free() {
		for (value in cachedShaders.values) {
			value.raw.free()
		}
	}
}