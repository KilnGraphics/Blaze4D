package me.hydos.rosella.render.util

import org.joml.Matrix4f
import org.joml.Vector2f
import org.joml.Vector3f
import org.joml.Vector4f
import org.lwjgl.vulkan.KHRSurface
import org.lwjgl.vulkan.VK10
import org.lwjgl.vulkan.VK11
import kotlin.reflect.KClass

private val errorMap = mutableMapOf<Int, String>().apply {
	this[VK10.VK_NOT_READY] = "VK_NOT_READY"
	this[VK10.VK_TIMEOUT] = "VK_TIMEOUT"
	this[VK10.VK_EVENT_SET] = "VK_EVENT_SET"
	this[VK10.VK_EVENT_RESET] = "VK_EVENT_RESET"
	this[VK10.VK_INCOMPLETE] = "VK_INCOMPLETE"
	this[VK10.VK_ERROR_OUT_OF_HOST_MEMORY] = "VK_ERROR_OUT_OF_HOST_MEMORY"
	this[VK11.VK_ERROR_OUT_OF_POOL_MEMORY] = "VK_ERROR_OUT_OF_POOL_MEMORY"
	this[VK10.VK_ERROR_OUT_OF_DEVICE_MEMORY] = "VK_ERROR_OUT_OF_DEVICE_MEMORY"
	this[VK10.VK_ERROR_INITIALIZATION_FAILED] = "VK_ERROR_INITIALIZATION_FAILED"
	this[VK10.VK_ERROR_DEVICE_LOST] = "VK_ERROR_DEVICE_LOST"
	this[VK10.VK_ERROR_MEMORY_MAP_FAILED] = "VK_ERROR_MEMORY_MAP_FAILED"
	this[VK10.VK_ERROR_LAYER_NOT_PRESENT] = "VK_ERROR_LAYER_NOT_PRESENT"
	this[VK10.VK_ERROR_EXTENSION_NOT_PRESENT] = "VK_ERROR_EXTENSION_NOT_PRESENT"
	this[VK10.VK_ERROR_FEATURE_NOT_PRESENT] = "VK_ERROR_FEATURE_NOT_PRESENT"
	this[VK10.VK_ERROR_INCOMPATIBLE_DRIVER] = "VK_ERROR_INCOMPATIBLE_DRIVER"
	this[VK10.VK_ERROR_TOO_MANY_OBJECTS] = "VK_ERROR_TOO_MANY_OBJECTS"
	this[VK10.VK_ERROR_FORMAT_NOT_SUPPORTED] = "VK_ERROR_FORMAT_NOT_SUPPORTED"
	this[VK10.VK_ERROR_FRAGMENTED_POOL] = "VK_ERROR_FRAGMENTED_POOL"
	this[VK10.VK_ERROR_UNKNOWN] = "VK_ERROR_UNKNOWN"
	this[KHRSurface.VK_ERROR_NATIVE_WINDOW_IN_USE_KHR] = "VK_ERROR_NATIVE_WINDOW_IN_USE_KHR"
}

private val SIZEOF_CACHE = mutableMapOf<Class<*>, Int>().apply {
	this[Byte::class.java] = Byte.SIZE_BYTES
	this[Character::class.java] = Character.BYTES
	this[Short::class.java] = Short.SIZE_BYTES
	this[Integer::class.java] = Integer.BYTES
	this[Float::class.java] = Float.SIZE_BYTES
	this[Long::class.java] = Long.SIZE_BYTES
	this[Double::class.java] = Double.SIZE_BYTES

	this[Vector2f::class.java] = 2 * Float.SIZE_BYTES
	this[Vector3f::class.java] = 3 * Float.SIZE_BYTES
	this[Vector4f::class.java] = 4 * Float.SIZE_BYTES

	this[Matrix4f::class.java] = 16 * java.lang.Float.BYTES
}

fun sizeof(obj: Any?): Int {
	return if (obj == null) 0 else SIZEOF_CACHE[obj.javaClass] ?: 0
}

fun sizeof(kClass: KClass<*>): Int {
	return SIZEOF_CACHE[kClass.java] ?: 0
}

fun alignof(obj: Any?): Int {
	return if (obj == null) 0 else SIZEOF_CACHE[obj.javaClass] ?: Integer.BYTES
}

fun alignas(offset: Int, alignment: Int): Int {
	return if (offset % alignment == 0) offset else (offset - 1 or alignment - 1) + 1
}

fun Int.ok(): Int {
	if (this != VK10.VK_SUCCESS) {
		throw RuntimeException(errorMap[this] ?: toString(16))
	}
	return this
}

fun Int.ok(message: String): Int {
	if (this != VK10.VK_SUCCESS) {
		throw RuntimeException(message + " Caused by: " + errorMap[this] + " (" + this + ")")
	}
	return this
}