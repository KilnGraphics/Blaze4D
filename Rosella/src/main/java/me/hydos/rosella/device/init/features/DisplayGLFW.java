package me.hydos.rosella.device.init.features;

import me.hydos.rosella.device.VulkanQueue;
import me.hydos.rosella.device.init.DeviceBuildConfigurator;
import me.hydos.rosella.device.init.DeviceBuildInformation;
import me.hydos.rosella.util.NamedID;

import java.util.concurrent.Future;

/**
 * Tests for display capabilities and allocates a display queue.
 *
 * The returned meta object will be the queue request.
 */
public class DisplayGLFW extends ApplicationFeature {

    public static final NamedID NAME = new NamedID("rosella:display_glfw");

    public DisplayGLFW() {
        super(NAME);
    }

    @Override
    public DisplayGLFWInstance createInstance() {
        return new DisplayGLFWInstance();
    }

    public class DisplayGLFWInstance extends ApplicationFeature.Instance {

        @Override
        public void testFeatureSupport(DeviceBuildInformation meta) {

        }

        @Override
        public Object enableFeature(DeviceBuildConfigurator meta) {
            return null;
        }
    }

    public record Meta(Future<VulkanQueue> queue) {
    }
}
