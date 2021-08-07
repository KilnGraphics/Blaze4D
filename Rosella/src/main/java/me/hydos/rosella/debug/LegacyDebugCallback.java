package me.hydos.rosella.debug;

import me.hydos.rosella.logging.DebugLogger;
import org.lwjgl.vulkan.VkDebugUtilsMessengerCallbackDataEXT;

public class LegacyDebugCallback extends VulkanDebugCallback.Callback {

    private DebugLogger logger;

    public LegacyDebugCallback(DebugLogger logger) {
        super(MessageSeverity.allBits(), MessageType.allBits());
        this.logger = logger;
    }

    @Override
    protected void callInternal(MessageSeverity severity, MessageType type, VkDebugUtilsMessengerCallbackDataEXT data) {
        String message = data.pMessageString();

        switch(type) {
            case GENERAL -> this.logger.logGeneral(message, severity);
            case VALIDATION -> this.logger.logValidation(message, severity);
            case PERFORMANCE -> this.logger.logPerformance(message, severity);
            default -> this.logger.logUnknown(message, severity);
        }
    }
}
