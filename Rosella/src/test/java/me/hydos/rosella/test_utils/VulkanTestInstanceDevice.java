package me.hydos.rosella.test_utils;

import me.hydos.rosella.device.VulkanDevice;
import me.hydos.rosella.device.init.DeviceBuilder;
import me.hydos.rosella.device.init.InitializationRegistry;
import me.hydos.rosella.device.init.InstanceBuilder;
import me.hydos.rosella.device.init.VulkanInstance;
import org.lwjgl.vulkan.VK10;

public class VulkanTestInstanceDevice implements AutoCloseable {

    public final VulkanInstance instance;
    public final VulkanDevice device;

    public VulkanTestInstanceDevice(InitializationRegistry registry) {
        InstanceBuilder instanceBuilder = new InstanceBuilder(registry);
        this.instance = instanceBuilder.build("RosellaTests", VK10.VK_MAKE_VERSION(1, 0, 0));

        DeviceBuilder deviceBuilder = new DeviceBuilder(this.instance, registry);
        this.device = deviceBuilder.build();
    }

    @Override
    public void close() {
        this.device.destroy();
        this.instance.destroy();
    }
}
