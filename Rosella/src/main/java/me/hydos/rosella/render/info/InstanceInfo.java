package me.hydos.rosella.render.info;

import me.hydos.rosella.Rosella;
import me.hydos.rosella.device.VulkanDevice;
import me.hydos.rosella.memory.Memory;
import me.hydos.rosella.memory.MemoryCloseable;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.shader.ubo.Ubo;
import me.hydos.rosella.render.texture.Texture;
import org.jetbrains.annotations.NotNull;

/**
 * Info such as the {@link Material} and {@link Ubo} for rendering objects
 */
public record InstanceInfo(Ubo ubo,
                           Texture[] textures,
                           Material material) implements MemoryCloseable {

    @Override
    public void free(VulkanDevice device, Memory memory) {
        ubo.free(device, memory);
        material.getShaderProgram().getDescriptorManager().freeDescriptorSets(ubo.getDescriptors());
    }

    /**
     * Called when Command Buffers need to be refreshed.
     *
     * @param rosella the Rosella
     */
    public void rebuild(@NotNull Rosella rosella) {
        if (ubo.getUniformBuffers().size() == 0) {
            ubo.create(rosella.renderer.swapchain);
            material.getShaderProgram().getDescriptorManager().createNewDescriptor(textures, ubo);
        }
    }

    /**
     * Called when the {@link me.hydos.rosella.render.swapchain.Swapchain} needs to be recreated. all {@link me.hydos.rosella.render.descriptorsets.DescriptorSets}'s will need to be recreated
     *
     * @param rosella the Rosella
     */
    public void hardRebuild(@NotNull Rosella rosella) {
        material.getShaderProgram().getDescriptorManager().clearDescriptorSets(ubo.getDescriptors());
        ubo.free(rosella.common.device, rosella.common.memory);

        if (ubo.getUniformBuffers().size() == 0) {
            ubo.create(rosella.renderer.swapchain);
        }

        material.getShaderProgram().getDescriptorManager().createNewDescriptor(textures, ubo);
    }
}
