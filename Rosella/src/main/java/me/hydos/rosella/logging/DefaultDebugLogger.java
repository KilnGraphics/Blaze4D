package me.hydos.rosella.logging;

import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

import static org.lwjgl.vulkan.VK10.VK_FALSE;

public class DefaultDebugLogger implements DebugLogger {

    private static final Logger LOGGER = LogManager.getLogger();

    @Override
    public int logValidation(String message, String severity) {
        LOGGER.error(message);
        return VK_FALSE;
    }

    @Override
    public int logPerformance(String message, String severity) {
        LOGGER.warn(message);
        return VK_FALSE;
    }

    @Override
    public int logGeneral(String message, String severity) {
        LOGGER.info(message);
        return VK_FALSE;
    }

    @Override
    public int logUnknown(String message, String severity) {
        LOGGER.info(message);
        return VK_FALSE;
    }
}
