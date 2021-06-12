package me.hydos.rosella.render.shader

import me.hydos.rosella.render.device.Device
import me.hydos.rosella.render.resource.Identifier

class ShaderManager(val device: Device) {

	var shaders = HashMap<Identifier, RawShaderProgram>()
	var cachedShaders = HashMap<Identifier, ShaderProgram>()

	fun getOrCreateShader(identifier: Identifier): ShaderProgram? {
		if(!cachedShaders.containsKey(identifier)) {
			cachedShaders[identifier] = ShaderProgram(shaders[identifier]!!, device)
		}

		return cachedShaders[identifier]
	}

	@Deprecated("Try not to store instances of a raw shader program. only store an Identifier or store an ShaderProgram")
	fun getOrCreateShader(rawShader: RawShaderProgram): ShaderProgram? {
		for (identifier in shaders.keys) {
			if(rawShader == shaders[identifier]) {
				return getOrCreateShader(identifier)
			}
		}
		error("Couldn't find a loaded shader matching that. oh no")
	}
}