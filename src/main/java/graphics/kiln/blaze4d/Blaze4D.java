package graphics.kiln.blaze4d;

import graphics.kiln.blaze4d.core.Blaze4DCore;

import graphics.kiln.blaze4d.core.Frame;
import graphics.kiln.blaze4d.core.natives.Natives;
import graphics.kiln.blaze4d.core.types.B4DMeshData;
import graphics.kiln.blaze4d.core.types.B4DUniformData;
import net.fabricmc.api.ClientModInitializer;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;
import org.apache.logging.log4j.message.StringFormatterMessageFactory;

public class Blaze4D implements ClientModInitializer {

    public static final Logger LOGGER = LogManager.getLogger("Blaze4D", new StringFormatterMessageFactory());

    public static Blaze4DCore core;
    public static Frame currentFrame;
    public static long glfwWindow;

    public static void pushUniform(long shaderId, B4DUniformData data) {
        if(currentFrame != null) {
            currentFrame.updateUniform(shaderId, data);
        } else {
            LOGGER.warn("Updated uniform outside of frame");
        }
    }

    public static void drawImmediate(long shaderId, B4DMeshData data) {
        if(currentFrame != null) {
            currentFrame.drawImmediate(shaderId, data);
        } else {
            LOGGER.warn("Attempted to draw outside of frame");
        }
    }

    @Override
    public void onInitializeClient() {
        Natives.verifyInit();
        if(System.getProperty("b4d.enable_renderdoc") != null) {
            System.loadLibrary("renderdoc");
        }
    }
}
