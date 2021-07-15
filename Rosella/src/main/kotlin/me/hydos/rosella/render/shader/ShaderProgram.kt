package me.hydos.rosella.render.shader

import me.hydos.rosella.Rosella
import me.hydos.rosella.device.VulkanDevice
import me.hydos.rosella.render.util.compileShaderFile
import me.hydos.rosella.render.util.ok
import me.hydos.rosella.ubo.DescriptorManager
import org.lwjgl.system.MemoryStack
import org.lwjgl.vulkan.VK10
import org.lwjgl.vulkan.VkShaderModuleCreateInfo
import java.nio.ByteBuffer

class ShaderProgram(val raw: RawShaderProgram, val rosella: Rosella, maxObjects: Int) {

    private val fragmentShader by lazy {
        compileShaderFile(raw.fragmentShader!!, ShaderType.FRAGMENT_SHADER).also {
            fragmentShaderCompiled = true
        }
    }
    private val vertexShader by lazy {
        compileShaderFile(raw.vertexShader!!, ShaderType.VERTEX_SHADER).also {
            vertexShaderCompiled = true
        }
    }
    val descriptorManager = DescriptorManager(maxObjects, this, rosella.renderer.swapchain, rosella.common.device, rosella.common.memory)

    private var fragmentShaderCompiled: Boolean = false
    private var vertexShaderCompiled: Boolean = false

    /**
     * Create a Vulkan shader module. used during pipeline creation.
     */
    private fun createShader(spirvCode: ByteBuffer, device: VulkanDevice): Long {
        MemoryStack.stackPush().use { stack ->
            val createInfo = VkShaderModuleCreateInfo.callocStack(stack)
                .sType(VK10.VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO)
                .pCode(spirvCode)
            val pShaderModule = stack.mallocLong(1)
            VK10.vkCreateShaderModule(device.rawDevice, createInfo, null, pShaderModule).ok()
            return pShaderModule[0]
        }
    }

    fun getVertShaderModule(): Long {
        return createShader(vertexShader.bytecode(), rosella.common.device)
    }

    fun getFragShaderModule(): Long {
        return createShader(fragmentShader.bytecode(), rosella.common.device)
    }

    /**
     * Free Shaders
     */
    fun free() {
        if (vertexShaderCompiled) vertexShader.free()
        if (fragmentShaderCompiled) fragmentShader.free()
        raw.free()
    }
}
