package me.hydos.rosella.test_utils;

import me.hydos.rosella.device.init.InitializationRegistry;
import me.hydos.rosella.device.init.InstanceBuilder;
import me.hydos.rosella.device.init.VulkanInstance;
import org.lwjgl.vulkan.VK10;

public class VulkanTestInstance implements AutoCloseable {

    public final VulkanInstance instance;

    public VulkanTestInstance(InitializationRegistry registry) {
        InstanceBuilder instanceBuilder = new InstanceBuilder(registry);
        this.instance = instanceBuilder.build("RosellaTests", VK10.VK_MAKE_VERSION(1, 0, 0));
    }

    @Override
    public void close() {
        this.instance.destroy();
    }
}
