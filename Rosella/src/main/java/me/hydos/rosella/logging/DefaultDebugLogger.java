package me.hydos.rosella.logging;

import me.hydos.rosella.debug.MessageSeverity;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

import static org.lwjgl.vulkan.VK10.VK_FALSE;

public class DefaultDebugLogger implements DebugLogger {

    private static final Logger LOGGER = LogManager.getLogger("Vulkan");

    @Override
    public int logValidation(String message, MessageSeverity severity) {
        LOGGER.error(severity + ": " + message);
        return VK_FALSE;
    }

    @Override
    public int logPerformance(String message, MessageSeverity severity) {
        LOGGER.warn(severity + ": " + message);
        return VK_FALSE;
    }

    @Override
    public int logGeneral(String message, MessageSeverity severity) {
        LOGGER.info(severity + ": " + message);
        return VK_FALSE;
    }

    @Override
    public int logUnknown(String message, MessageSeverity severity) {
        LOGGER.info(severity + ": " + message);
        return VK_FALSE;
    }
}
