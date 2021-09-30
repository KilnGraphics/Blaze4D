package me.hydos.blaze4d;

import graphics.kiln.rosella.init.features.TriangleFan;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import graphics.kiln.rosella.Rosella;
import graphics.kiln.rosella.display.GlfwWindow;
import graphics.kiln.rosella.scene.object.impl.SimpleObjectManager;
import net.fabricmc.api.ClientModInitializer;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;
import org.apache.logging.log4j.message.StringFormatterMessageFactory;

public class Blaze4D implements ClientModInitializer {

    public static final Logger LOGGER = LogManager.getLogger("Blaze4D", new StringFormatterMessageFactory());
    public static final boolean VALIDATION_ENABLED = Boolean.parseBoolean(System.getProperty("rosella:validation"));
    public static final boolean RENDERDOC_ENABLED = Boolean.parseBoolean(System.getProperty("rosella:renderdoc"));

    public static Rosella rosella;
    public static GlfwWindow window;

    public static void finishSetup() {
        GlobalRenderSystem.emulateTriangleFans = !rosella.common.device.isFeatureEnabled(TriangleFan.NAME);
        rosella.renderer.rebuildCommandBuffers(rosella.renderer.mainRenderPass, rosella.common.fboManager.getActiveFbo());
    }

    @Override
    public void onInitializeClient() {
        if (RENDERDOC_ENABLED) {
            System.loadLibrary("renderdoc");
        }
    }
}
