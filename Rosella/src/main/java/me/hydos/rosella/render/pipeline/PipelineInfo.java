package me.hydos.rosella.render.pipeline;

import me.hydos.rosella.device.VulkanDevice;
import me.hydos.rosella.memory.Memory;
import me.hydos.rosella.memory.MemoryCloseable;

public record PipelineInfo(long pipelineLayout, long graphicsPipeline) implements MemoryCloseable {

    @Override
    public void free(VulkanDevice device, Memory memory) {
        memory.freePipeline(this);
    }
}
