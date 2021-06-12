package me.hydos.rosella.render.util

import me.hydos.rosella.render.resource.Resource
import org.lwjgl.system.MemoryUtil.NULL
import org.lwjgl.system.NativeResource
import org.lwjgl.util.shaderc.Shaderc.*
import java.nio.ByteBuffer

fun compileShaderFile(shader: Resource, shaderType: ShaderType): SpirV {
	val source = shader.openStream().readBytes().decodeToString()
	return compileShader(shader.identifier.file, source, shaderType)
}

fun compileShader(filename: String, source: String, shaderType: ShaderType): SpirV {
	val compiler = shaderc_compiler_initialize()

	if (compiler == NULL) {
		throw RuntimeException("Failed to create shader compiler")
	}

	val result: Long = shaderc_compile_into_spv(compiler, source, shaderType.kind, filename, "main", NULL)

	if (result == NULL) {
		throw RuntimeException("Failed to compile shader $filename into SPIR-V")
	}

	if (shaderc_result_get_compilation_status(result) != shaderc_compilation_status_success) {
		error("Failed to compile shader $filename into SPIR-V: ${shaderc_result_get_error_message(result)}")
	}

	shaderc_compiler_release(compiler)

	return SpirV(result, shaderc_result_get_bytes(result))
}

class SpirV(private val handle: Long, private var bytecode: ByteBuffer?) : NativeResource {
	fun bytecode(): ByteBuffer {
		return bytecode!!
	}

	override fun free() {
		shaderc_result_release(handle)
		bytecode = null
	}
}
