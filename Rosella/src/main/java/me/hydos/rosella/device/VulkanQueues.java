package me.hydos.rosella.device;

import me.hydos.rosella.vkobjects.VkCommon;
import org.lwjgl.PointerBuffer;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.vulkan.VkQueue;

import static org.lwjgl.vulkan.VK10.VK_NULL_HANDLE;
import static org.lwjgl.vulkan.VK10.vkGetDeviceQueue;

/**
 * The presentation and graphics queue used in {@link me.hydos.rosella.Rosella}
 */
public class VulkanQueues {

    public final VulkanQueue graphicsQueue;
    public final VulkanQueue presentQueue;

    public VulkanQueues(VulkanQueue graphicsQueue, VulkanQueue presentQueue) {
        this.graphicsQueue = graphicsQueue;
        this.presentQueue = presentQueue;
    }
}
