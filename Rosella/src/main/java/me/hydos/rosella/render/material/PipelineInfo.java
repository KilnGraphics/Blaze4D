package me.hydos.rosella.render.material;

import me.hydos.rosella.device.VulkanDevice;
import org.lwjgl.vulkan.VK10;

public record PipelineInfo(long pipelineLayout, long graphicsPipeline) {

    public void free(VulkanDevice device) {
        VK10.vkDestroyPipeline(device.rawDevice, graphicsPipeline, null);
        VK10.vkDestroyPipelineLayout(device.rawDevice, pipelineLayout, null);
    }
}
