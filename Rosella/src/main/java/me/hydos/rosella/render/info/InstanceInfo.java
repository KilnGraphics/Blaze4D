package me.hydos.rosella.render.info;

import me.hydos.rosella.Rosella;
import me.hydos.rosella.memory.MemoryCloseable;
import me.hydos.rosella.render.RosellaVk;
import me.hydos.rosella.render.device.Device;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.shader.ubo.Ubo;
import me.hydos.rosella.render.util.memory.Memory;
import org.jetbrains.annotations.NotNull;

/**
 * Info such as the {@link Material} and {@link Ubo} for rendering objects
 */
public class InstanceInfo implements MemoryCloseable {

    public Ubo ubo;
    public Material material;

    public InstanceInfo(Ubo ubo, Material material) {
        this.ubo = ubo;
        this.material = material;
    }

    @Override
    public void free(Device device, Memory memory) {
        ubo.free(device, memory);
        material.shader.getDescriptorManager().freeDescriptorSet(ubo.getDescriptors());
    }

    /**
     * Called when Command Buffers need to be refreshed. all {@link me.hydos.rosella.render.descriptorsets.DescriptorSet}'s will need to be recreated
     *
     * @param rosella The active instance of the Renderer
     */
    public void rebuild(@NotNull Rosella rosella) {
        material.shader.getDescriptorManager().freeDescriptorSet(ubo.getDescriptors());
        if (ubo.getUniformBuffers().size() == 0) {
            ubo.create(rosella.getRenderer().swapchain);
        }

        RosellaVk.prepareTextureForRender(rosella.getRenderer(), material.texture.getTextureImage().getTextureImage(), material.getImgFormat());

        material.shader.getDescriptorManager().createNewDescriptor(material.texture, ubo);
    }
}
