package me.hydos.rosella.debug;

import org.lwjgl.vulkan.EXTDebugUtils;

public enum MessageSeverity {
    VERBOSE(EXTDebugUtils.VK_DEBUG_UTILS_MESSAGE_SEVERITY_VERBOSE_BIT_EXT, "VERBOSE"),
    INFO(EXTDebugUtils.VK_DEBUG_UTILS_MESSAGE_SEVERITY_INFO_BIT_EXT, "INFO"),
    WARNING(EXTDebugUtils.VK_DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT, "WARNING"),
    ERROR(EXTDebugUtils.VK_DEBUG_UTILS_MESSAGE_SEVERITY_ERROR_BIT_EXT, "ERROR");

    public final int bits;
    public final String name;

    MessageSeverity(int bits, String name) {
        this.bits = bits;
        this.name = name;
    }

    public boolean isInMask(int mask) {
        return (mask & this.bits) == this.bits;
    }

    @Override
    public String toString() {
        return this.name;
    }

    public static int allBits() {
        return VERBOSE.bits | INFO.bits | WARNING.bits | ERROR.bits;
    }

    public static MessageSeverity fromBits(int bits) {
        return switch(bits) {
            case EXTDebugUtils.VK_DEBUG_UTILS_MESSAGE_SEVERITY_VERBOSE_BIT_EXT -> VERBOSE;
            case EXTDebugUtils.VK_DEBUG_UTILS_MESSAGE_SEVERITY_INFO_BIT_EXT -> INFO;
            case EXTDebugUtils.VK_DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT -> WARNING;
            case EXTDebugUtils.VK_DEBUG_UTILS_MESSAGE_SEVERITY_ERROR_BIT_EXT -> ERROR;
            default -> throw new RuntimeException("Bits are either a combination of bits or invalid");
        };
    }
}
