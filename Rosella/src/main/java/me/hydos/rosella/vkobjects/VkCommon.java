package me.hydos.rosella.vkobjects;

import me.hydos.rosella.device.VulkanDevice;
import me.hydos.rosella.device.VulkanQueues;
import me.hydos.rosella.display.Display;

/**
 * Common fields shared within the {@link me.hydos.rosella.Rosella} instance. sharing this info with other instances of the engine is extremely unsafe.
 */
public class VkCommon {

    /**
     * The display used to display the window.
     */
    public Display display;

    /**
     * The instance of vulkan and the debug logger.
     */
    public VulkanInstance vkInstance;

    /**
     * The surface of what we are displaying to. In general it will be a GLFW window surface.
     */
    public long surface;

    /**
     * The logical and physical device. used in most Vulkan calls.
     */
    public VulkanDevice device;

    /**
     * The Presentation and Graphics queue.
     */
    public VulkanQueues queues;
}
