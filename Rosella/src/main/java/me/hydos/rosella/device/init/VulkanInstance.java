package me.hydos.rosella.device.init;

import me.hydos.rosella.debug.VulkanDebugCallback;
import org.jetbrains.annotations.Nullable;
import org.lwjgl.vulkan.VK10;
import org.lwjgl.vulkan.VKCapabilitiesInstance;
import org.lwjgl.vulkan.VkInstance;

public class VulkanInstance {

    private final VkInstance instance;
    private final VulkanVersion version;
    private final VulkanDebugCallback debugCallback;

    public VulkanInstance(VkInstance instance) {
        this.instance = instance;
        this.version = VulkanVersion.fromVersionNumber(instance.getCapabilities().apiVersion);
        this.debugCallback = null;
    }

    public VulkanInstance(VkInstance instance, @Nullable VulkanDebugCallback callback) {
        this.instance = instance;
        this.version = VulkanVersion.fromVersionNumber(instance.getCapabilities().apiVersion);
        this.debugCallback = callback;
    }

    public VkInstance getInstance() {
        return this.instance;
    }

    public VKCapabilitiesInstance getCapabilities() {
        return instance.getCapabilities();
    }

    public VulkanVersion getVersion() {
        return this.version;
    }

    public void destroy() {
        if(this.debugCallback != null) {
            this.debugCallback.destroy();
        }
        VK10.vkDestroyInstance(this.instance, null);
    }
}
