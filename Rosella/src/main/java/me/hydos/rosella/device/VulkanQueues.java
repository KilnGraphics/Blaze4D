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

    public VulkanQueues(VkCommon common) {
        try (MemoryStack stack = MemoryStack.stackPush()) {
            PointerBuffer pQueues = stack.pointers(VK_NULL_HANDLE);
            vkGetDeviceQueue(common.device.rawDevice, common.device.indices.graphicsFamily, 0, pQueues);
            this.graphicsQueue = new VulkanQueue(new VkQueue((pQueues.get(0)), common.device.rawDevice), common.device.indices.graphicsFamily);

            // Need to make sure that if they are the same queue they are the same object and share the lock
            if(!common.device.indices.presentFamily.equals(common.device.indices.graphicsFamily)) {
                vkGetDeviceQueue(common.device.rawDevice, common.device.indices.presentFamily, 0, pQueues);
                this.presentQueue = new VulkanQueue(new VkQueue((pQueues.get(0)), common.device.rawDevice), common.device.indices.presentFamily);
            } else {
                this.presentQueue = this.graphicsQueue;
            }
        }
    }
}
