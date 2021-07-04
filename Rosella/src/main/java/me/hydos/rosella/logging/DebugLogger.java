package me.hydos.rosella.logging;

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
    int logValidation(String message, String severity);

    /**
     * Called when the driver logs a performance warning.
     *
     * @param message  the message sent from the driver
     * @param severity Severity can either be "VERBOSE", "INFO", "WARNING", or "ERROR"
     * @return Only VK_TRUE and VK_FALSE is allowed to be returned. anything else will be undocumented behaviour.
     */
    int logPerformance(String message, String severity);

    /**
     * Called when the driver logs a general message.
     *
     * @param message  the message sent from the driver
     * @param severity Severity can either be "VERBOSE", "INFO", "WARNING", or "ERROR"
     * @return Only VK_TRUE and VK_FALSE is allowed to be returned. anything else will be undocumented behaviour.
     */
    int logGeneral(String message, String severity);

    /**
     * Called when the driver logs a message we failed to parse.
     *
     * @param message  the message sent from the driver
     * @param severity Severity can either be "VERBOSE", "INFO", "WARNING", or "ERROR"
     * @return Only VK_TRUE and VK_FALSE is allowed to be returned. anything else will be undocumented behaviour.
     */
    int logUnknown(String message, String severity);
}
