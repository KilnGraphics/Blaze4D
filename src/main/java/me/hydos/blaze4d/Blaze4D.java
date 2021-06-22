package me.hydos.blaze4d;

import me.hydos.blaze4d.api.Materials;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.render.io.Window;
import net.fabricmc.api.ModInitializer;
import org.apache.logging.log4j.Level;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

public class Blaze4D implements ModInitializer {
    public static final Logger LOGGER = LogManager.getLogger("Blaze4D Render System");

    public static Rosella rosella;
    public static Window window;

    public static void finishAndRender() {
        rosella.getRenderer().rebuildCommandBuffers(rosella.getRenderer().renderPass, rosella);
        window.onMainLoop(() -> rosella.getRenderer().render(rosella));
    }

    @Override
    public void onInitialize() {
        ((org.apache.logging.log4j.core.Logger) LOGGER).setLevel(Level.ALL);
        try {
            System.loadLibrary("renderdoc");
        } catch (UnsatisfiedLinkError e) {
            LOGGER.warn("Unable to find renderdoc on path.");
        }
    }
}
