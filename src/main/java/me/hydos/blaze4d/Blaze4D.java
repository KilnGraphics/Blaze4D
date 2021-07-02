package me.hydos.blaze4d;

import me.hydos.rosella.Rosella;
import me.hydos.rosella.render.io.Window;
import net.fabricmc.api.ClientModInitializer;
import org.apache.logging.log4j.Level;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;
import org.apache.logging.log4j.message.StringFormatterMessageFactory;


public class Blaze4D implements ClientModInitializer {
    public static final Logger LOGGER = LogManager.getLogger("Blaze4D", new StringFormatterMessageFactory());
    public static final boolean VALIDATION_ENABLED = true;
    public static final boolean RENDERDOC_ENABLED = false;

    public static Rosella rosella;
    public static Window window;

    public static void finishAndRender() {
        rosella.getRenderer().rebuildCommandBuffers(rosella.getRenderer().renderPass, rosella);
    }

    @Override
    public void onInitializeClient() {
        ((org.apache.logging.log4j.core.Logger) LOGGER).setLevel(Level.ALL);
//        Configuration.DEBUG_MEMORY_ALLOCATOR.set(true);

        try {
            if (RENDERDOC_ENABLED) {
                System.loadLibrary("renderdoc");
            }
        } catch (UnsatisfiedLinkError e) {
            LOGGER.warn("Unable to find renderdoc on path.");
        }

        AftermathHandler.initialize();
    }
}
