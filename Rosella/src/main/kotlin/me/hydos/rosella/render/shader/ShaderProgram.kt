package me.hydos.rosella.render.shader

import me.hydos.rosella.render.device.Device
import me.hydos.rosella.render.util.ShaderType
import me.hydos.rosella.render.util.SpirV
import me.hydos.rosella.render.util.compileShaderFile
import me.hydos.rosella.render.util.ok
import org.lwjgl.system.MemoryStack
import org.lwjgl.vulkan.VK10
import org.lwjgl.vulkan.VkShaderModuleCreateInfo
import java.nio.ByteBuffer

class ShaderProgram(val raw: RawShaderProgram, val device: Device) {

	private val fragmentShader by lazy { compileShaderFile(raw.fragmentShader, ShaderType.FRAGMENT_SHADER) }
	private val vertexShader by lazy { compileShaderFile(raw.vertexShader, ShaderType.VERTEX_SHADER) }

	/**
	 * Create a Vulkan shader module. used during pipeline creation.
	 */
	private fun createShader(spirvCode: ByteBuffer, device: Device): Long {
		MemoryStack.stackPush().use { stack ->
			val createInfo = VkShaderModuleCreateInfo.callocStack(stack)
				.sType(VK10.VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO)
				.pCode(spirvCode)
			val pShaderModule = stack.mallocLong(1)
			VK10.vkCreateShaderModule(device.device, createInfo, null, pShaderModule).ok()
			return pShaderModule[0]
		}
	}

	fun getVertShaderModule(): Long {
		return createShader(vertexShader.bytecode(), device)
	}

	fun getFragShaderModule(): Long {
		return createShader(fragmentShader.bytecode(), device)
	}

	/**
	 * Free Shaders
	 */
	fun free() {
		vertexShader.free()
		fragmentShader.free()
	}
}