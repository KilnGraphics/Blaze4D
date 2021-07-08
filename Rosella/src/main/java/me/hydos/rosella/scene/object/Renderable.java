package me.hydos.rosella.scene.object;

import me.hydos.rosella.Rosella;
import me.hydos.rosella.device.VulkanDevice;
import me.hydos.rosella.render.info.InstanceInfo;
import me.hydos.rosella.render.info.RenderInfo;
import me.hydos.rosella.render.renderer.Renderer;
import me.hydos.rosella.memory.Memory;
import me.hydos.rosella.vkobjects.VkCommon;

/**
 * Contains data for what you want to render
 */
public interface Renderable {

    /**
     * Called when the Application asked {@link Rosella} to add this to the scene.
     *
     * @param rosella the common fields used by {@link Rosella}
     */
    void onAddedToScene(Rosella rosella);

    /**
     * Called when an object's memory can be freed' safely
     *
     * @param memory the rosella Memory Manager
     * @param device the Device rosella is rendering on
     */
    void free(Memory memory, VulkanDevice device);

    /**
     * Called when the swapchain needs to be resized
     * @param rosella the instance of the {@link Rosella} engine used.
     */
    void rebuild(Rosella rosella);

    InstanceInfo getInstanceInfo();

    RenderInfo getRenderInfo();
}
