package me.hydos.rosella.device.init;

import me.hydos.rosella.device.VulkanQueue;
import org.lwjgl.vulkan.VkPhysicalDeviceFeatures;

import java.util.concurrent.Future;

public interface DeviceBuildConfigurator extends DeviceBuildInformation {

    /**
     * Adds a new queue request
     *
     * @param family The family that is requested.
     * @return A Future which will contain the VulkanQueue after the device has been created.
     */
    Future<VulkanQueue> addQueueRequest(int family);

    /**
     * Adds a extension to the set of enabled extensions. This function does not validate if the extension
     * is actually supported.
     *
     * @param extension The name of the extension.
     */
    void enableExtension(String extension);

    /**
     * Returns an instance that can be used to configure device features.
     *
     * @return A VkPhysicalDeviceProperties instance to configure the device.
     */
    VkPhysicalDeviceFeatures configureDeviceFeatures();
}
