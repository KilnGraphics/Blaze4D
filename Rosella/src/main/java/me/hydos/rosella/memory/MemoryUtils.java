package me.hydos.rosella.memory;

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
public class MemoryUtils {

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
            entry(Matrix4f.class, 16 * Float.BYTES)
    );

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
