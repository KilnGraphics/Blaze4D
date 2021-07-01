package me.hydos.rosella.render.object;

import me.hydos.rosella.Rosella;
import me.hydos.rosella.render.device.Device;
import me.hydos.rosella.render.info.InstanceInfo;
import me.hydos.rosella.render.info.RenderInfo;
import me.hydos.rosella.render.util.memory.Memory;

/**
 * Contains data for what you want to render
 */
public interface Renderable {

    /**
     * Called when the Application asked {@link Rosella} to add this to the scene.
     *
     * @param rosella the instance of the {@link Rosella} engine used.
     */
    void onAddedToScene(Rosella rosella);

    /**
     * Called when an object's memory can be freed' safely
     *
     * @param memory the rosella Memory Manager
     * @param device the Device rosella is rendering on
     */
    void free(Memory memory, Device device);

    /**
     * Called when the swapchain needs to be resized
     * @param rosella the instance of the {@link Rosella} engine used.
     */
    void rebuild(Rosella rosella);

    default boolean isReady() {
        return getRenderInfo().areBuffersAllocated();
    }

    InstanceInfo getInstanceInfo();

    RenderInfo getRenderInfo();
}
