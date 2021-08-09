package me.hydos.blaze4d;

import me.hydos.blaze4d.api.GlobalRenderSystem;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.device.init.features.TriangleFan;
import me.hydos.rosella.display.GlfwWindow;
import me.hydos.rosella.scene.object.impl.SimpleObjectManager;
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
        GlobalRenderSystem.emulateTriangleFans = !rosella.vulkanDevice.isFeatureEnabled(TriangleFan.NAME);
        rosella.renderer.rebuildCommandBuffers(rosella.renderer.mainRenderPass, (SimpleObjectManager) rosella.objectManager);
    }

    @Override
    public void onInitializeClient() {
        if (RENDERDOC_ENABLED) {
            System.loadLibrary("renderdoc");
        }
    }
}
