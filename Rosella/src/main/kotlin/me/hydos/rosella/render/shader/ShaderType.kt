package me.hydos.rosella.render.shader

import org.lwjgl.util.shaderc.Shaderc
import org.lwjgl.vulkan.VK10

enum class ShaderType(val shaderCType: Int, val vkShaderStage: Int) {
    VERTEX_SHADER(Shaderc.shaderc_glsl_vertex_shader, VK10.VK_SHADER_STAGE_VERTEX_BIT),
    FRAGMENT_SHADER(Shaderc.shaderc_glsl_fragment_shader, VK10.VK_SHADER_STAGE_FRAGMENT_BIT);
}
