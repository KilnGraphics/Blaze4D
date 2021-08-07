package me.hydos.rosella.logging;

import me.hydos.rosella.debug.MessageSeverity;

/**
 * Interface used for handling the messages sent back from the driver
 */
public interface DebugLogger {

    /**
     * Called when the driver logs a validation message.
     *
     * @param message  the message sent from the driver
     * @param severity Severity can either be "VERBOSE", "INFO", "WARNING", or "ERROR"
     * @return Only VK_TRUE and VK_FALSE is allowed to be returned. anything else will be undocumented behaviour.
     */
    int logValidation(String message, MessageSeverity severity);

    /**
     * Called when the driver logs a performance warning.
     *
     * @param message  the message sent from the driver
     * @param severity Severity can either be "VERBOSE", "INFO", "WARNING", or "ERROR"
     * @return Only VK_TRUE and VK_FALSE is allowed to be returned. anything else will be undocumented behaviour.
     */
    int logPerformance(String message, MessageSeverity severity);

    /**
     * Called when the driver logs a general message.
     *
     * @param message  the message sent from the driver
     * @param severity Severity can either be "VERBOSE", "INFO", "WARNING", or "ERROR"
     * @return Only VK_TRUE and VK_FALSE is allowed to be returned. anything else will be undocumented behaviour.
     */
    int logGeneral(String message, MessageSeverity severity);

    /**
     * Called when the driver logs a message we failed to parse.
     *
     * @param message  the message sent from the driver
     * @param severity Severity can either be "VERBOSE", "INFO", "WARNING", or "ERROR"
     * @return Only VK_TRUE and VK_FALSE is allowed to be returned. anything else will be undocumented behaviour.
     */
    int logUnknown(String message, MessageSeverity severity);
}
