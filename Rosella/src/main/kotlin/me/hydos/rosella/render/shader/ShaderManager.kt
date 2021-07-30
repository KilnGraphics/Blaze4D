package me.hydos.rosella.render.shader

import me.hydos.rosella.Rosella

class ShaderManager(val rosella: Rosella) {

    var cachedShaders = HashMap<RawShaderProgram, ShaderProgram>()

    fun getOrCreateShader(rawShader: RawShaderProgram): ShaderProgram? {
        if (!cachedShaders.containsKey(rawShader)) {
            cachedShaders[rawShader] = ShaderProgram(rawShader, rosella, rawShader.maxObjCount)
            // TODO: not sure if this should actually be here or later
            rawShader.createDescriptorSetLayout()
        }

        return cachedShaders[rawShader]
    }

    fun free() {
        for (program in cachedShaders.values) {
            program.free()
        }
    }
}
