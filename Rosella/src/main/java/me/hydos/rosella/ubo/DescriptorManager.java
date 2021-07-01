package me.hydos.rosella.ubo;

import me.hydos.rosella.render.descriptorsets.DescriptorSet;
import me.hydos.rosella.render.device.Device;
import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.shader.ubo.Ubo;
import me.hydos.rosella.render.swapchain.Swapchain;
import me.hydos.rosella.render.texture.Texture;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

/**
 * Manages Descriptor Sets. Allows them to be reused.
 */
public class DescriptorManager {

    private static final Logger LOGGER = LogManager.getLogger("DescriptorManager");
    private final ShaderProgram program;
    private final Swapchain swapchain;
    private final Device device;
    private final int maxObjects;
    private int activeDescriptorCount;

    /**
     * Creates a new {@link DescriptorManager} object
     *
     * @param maxObjects the max amount of DescriptorSet's
     * @param program    the {@link ShaderProgram} to base it off
     */
    public DescriptorManager(int maxObjects, ShaderProgram program, Swapchain swapchain, Device device) {
        this.maxObjects = maxObjects;
        this.program = program;
        this.swapchain = swapchain;
        this.device = device;
    }

    /**
     * Allocates a new {@link DescriptorSet}. This should only be called when no free {@link DescriptorSet}'s are available
     *
     * @param texture the {@link Texture} to use with the {@link DescriptorSet}
     * @param ubo     the {@link Ubo} to use with the {@link DescriptorSet}
     */
    public void createNewDescriptor(Texture texture, Ubo ubo) {
        activeDescriptorCount++;
        if (maxObjects <= activeDescriptorCount) {
            throw new RuntimeException("Too many Descriptor Sets are being used at once (max is " + activeDescriptorCount + ")");
        }
        program.getRaw().createDescriptorSets(swapchain, LOGGER, texture, ubo);
    }

    public void freeDescriptorSet(DescriptorSet set) {
        set.free(device);
        activeDescriptorCount--;
    }
}
