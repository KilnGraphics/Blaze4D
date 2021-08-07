package me.hydos.rosella.device;

import me.hydos.rosella.util.NamedID;
import org.lwjgl.vulkan.VK10;
import org.lwjgl.vulkan.VkDevice;

import java.util.Collections;
import java.util.Map;

public class VulkanDevice {

    private final VkDevice device;
    private final Map<NamedID, Object> enableFeatures;

    public VulkanDevice(VkDevice device, Map<NamedID, Object> enabledFeatures) {
        this.device = device;
        this.enableFeatures = Collections.unmodifiableMap(enabledFeatures);
    }

    public VkDevice getDevice() {
        return this.device;
    }

    public void destroy() {
        VK10.vkDestroyDevice(this.device, null);
    }

    /**
     * Tests if a ApplicationFeature is enabled for this device.
     *
     * @param name The name of the feature.
     * @return True if the feature is enabled. False otherwise.
     */
    public boolean isFeatureEnabled(NamedID name) {
        return enableFeatures.containsKey(name);
    }

    /**
     * Retrieves the metadata for an enabled ApplicationFeature.
     *
     * @param name The name of the feature.
     * @return The metadata for the feature or null if the feature didnt generate any metadata.
     */
    public Object getFeatureMeta(NamedID name) {
        return enableFeatures.get(name);
    }
}
