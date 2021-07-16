package me.hydos.rosella.util;

import org.joml.Matrix4f;
import org.joml.Vector2f;
import org.joml.Vector3f;
import org.joml.Vector4f;
import org.lwjgl.vulkan.EXTDebugReport;
import org.lwjgl.vulkan.KHRSurface;
import org.lwjgl.vulkan.VK10;
import org.lwjgl.vulkan.VK11;

import java.util.Map;

import static java.util.Map.entry;
import static java.util.Map.ofEntries;

/**
 * Utils to work with vulkan. Not much else to say about it.
 */
public class VulkanUtils {

    private static final Map<Integer, String> ERROR_NAMES = ofEntries(
            entry(VK10.VK_NOT_READY, "VK_NOT_READY"),
            entry(VK10.VK_TIMEOUT, "VK_TIMEOUT"),
            entry(VK10.VK_EVENT_SET, "VK_EVENT_SET"),
            entry(VK10.VK_EVENT_RESET, "VK_EVENT_RESET"),
            entry(VK10.VK_INCOMPLETE, "VK_INCOMPLETE"),
            entry(VK10.VK_ERROR_OUT_OF_HOST_MEMORY, "VK_ERROR_OUT_OF_HOST_MEMORY"),
            entry(VK11.VK_ERROR_OUT_OF_POOL_MEMORY, "VK_ERROR_OUT_OF_POOL_MEMORY"),
            entry(VK10.VK_ERROR_OUT_OF_DEVICE_MEMORY, "VK_ERROR_OUT_OF_DEVICE_MEMORY"),
            entry(VK10.VK_ERROR_INITIALIZATION_FAILED, "VK_ERROR_INITIALIZATION_FAILED"),
            entry(VK10.VK_ERROR_DEVICE_LOST, "VK_ERROR_DEVICE_LOST"),
            entry(VK10.VK_ERROR_MEMORY_MAP_FAILED, "VK_ERROR_MEMORY_MAP_FAILED"),
            entry(VK10.VK_ERROR_LAYER_NOT_PRESENT, "VK_ERROR_LAYER_NOT_PRESENT"),
            entry(VK10.VK_ERROR_EXTENSION_NOT_PRESENT, "VK_ERROR_EXTENSION_NOT_PRESENT"),
            entry(VK10.VK_ERROR_FEATURE_NOT_PRESENT, "VK_ERROR_FEATURE_NOT_PRESENT"),
            entry(VK10.VK_ERROR_INCOMPATIBLE_DRIVER, "VK_ERROR_INCOMPATIBLE_DRIVER"),
            entry(VK10.VK_ERROR_TOO_MANY_OBJECTS, "VK_ERROR_TOO_MANY_OBJECTS"),
            entry(VK10.VK_ERROR_FORMAT_NOT_SUPPORTED, "VK_ERROR_FORMAT_NOT_SUPPORTED"),
            entry(VK10.VK_ERROR_FRAGMENTED_POOL, "VK_ERROR_FRAGMENTED_POOL"),
            entry(VK10.VK_ERROR_UNKNOWN, "VK_ERROR_UNKNOWN"),
            entry(KHRSurface.VK_ERROR_NATIVE_WINDOW_IN_USE_KHR, "VK_ERROR_NATIVE_WINDOW_IN_USE_KHR"),
            entry(EXTDebugReport.VK_ERROR_VALIDATION_FAILED_EXT, "VK_ERROR_VALIDATION_FAILED_EXT")
    );

    private static final Map<Class<?>, Integer> SIZE_MAP = ofEntries(
            entry(Byte.class, Byte.BYTES),
            entry(Character.class, Character.BYTES),
            entry(Short.class, Short.BYTES),
            entry(Integer.class, Integer.BYTES),
            entry(Float.class, Float.BYTES),
            entry(Long.class, Long.BYTES),
            entry(Double.class, Double.BYTES),
            entry(Vector2f.class, 2 * Float.BYTES),
            entry(Vector3f.class, 3 * Float.BYTES),
            entry(Vector4f.class, 4 * Float.BYTES),
            entry(Matrix4f.class, 16 * java.lang.Float.BYTES)
    );

    public static void ok(int vkResult) {
        if (vkResult != VK10.VK_SUCCESS) {
            throw new RuntimeException(ERROR_NAMES.getOrDefault(vkResult, Integer.toString(vkResult)));
        }
    }

    public static void ok(int vkResult, String message) {
        if (vkResult != VK10.VK_SUCCESS) {
            throw new RuntimeException(message + ", caused by " + ERROR_NAMES.getOrDefault(vkResult, Integer.toString(vkResult)));
        }
    }

    public static int size(Class<?> klass) {
        return SIZE_MAP.get(klass);
    }

    public static int alignment(Class<?> klass) {
        return SIZE_MAP.get(klass);
    }

    public static int align(int offset, int alignment) {
        if (offset % alignment == 0) {
            return offset;
        } else {
            return (offset - 1 | alignment - 1) + 1;
        }
    }
}
