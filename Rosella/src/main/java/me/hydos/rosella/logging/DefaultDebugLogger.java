package me.hydos.rosella.logging;

import me.hydos.rosella.Rosella;

import static org.lwjgl.vulkan.VK10.VK_FALSE;

public class DefaultDebugLogger implements DebugLogger {

    @Override
    public int logValidation(String message, String severity) {
        Rosella.LOGGER.error(message);
        return VK_FALSE;
    }

    @Override
    public int logPerformance(String message, String severity) {
        Rosella.LOGGER.warn(message);
        return VK_FALSE;
    }

    @Override
    public int logGeneral(String message, String severity) {
        Rosella.LOGGER.info(message);
        return VK_FALSE;
    }

    @Override
    public int logUnknown(String message, String severity) {
        Rosella.LOGGER.info(message);
        return VK_FALSE;
    }
}
