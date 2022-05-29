package graphics.kiln.blaze4d;

import net.fabricmc.api.ClientModInitializer;
import jdk.incubator.foreign.*;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;
import org.apache.logging.log4j.message.StringFormatterMessageFactory;
import org.lwjgl.glfw.GLFW;
import org.lwjgl.system.APIUtil;

public class Blaze4D implements ClientModInitializer {

    public static final Logger LOGGER = LogManager.getLogger("Blaze4D", new StringFormatterMessageFactory());
//    public static final boolean VALIDATION_ENABLED = Boolean.parseBoolean(System.getProperty("rosella:validation"));
//    public static final boolean RENDERDOC_ENABLED = Boolean.parseBoolean(System.getProperty("rosella:renderdoc"));
//
//    public static Rosella rosella;
//    public static GlfwWindow window;
//    public static FrameBufferObject mainFbo;

    public static void finishSetup() {
//        mainFbo = rosella.common.fboManager.getActiveFbo();
//        GlobalRenderSystem.emulateTriangleFans = !rosella.common.device.isFeatureEnabled(TriangleFan.NAME);
//        rosella.renderer.rebuildCommandBuffers(rosella.renderer.mainRenderPass);
    }

    @Override
    public void onInitializeClient() {
        Blaze4DNatives.load();

        int[] major = new int[]{ 0 };
        int[] minor = new int[]{ 0 };
        int[] patch = new int[]{ 0 };
        GLFW.glfwGetVersion(major, minor, patch);

        LOGGER.error("GLFW VERSION: " + major[0] + "." + minor[0] + "." + patch[0]);

        /*try {
            Blaze4DNatives.b4dPreInitGlfw.invokeExact(
                    MemoryAddress.ofLong(APIUtil.apiGetFunctionAddress(GLFW.getLibrary(), "glfwInitVulkanLoader"))
            );
        } catch(Throwable ex) {
            throw new RuntimeException(ex);
        }*/

//        if (RENDERDOC_ENABLED) {
//            System.loadLibrary("renderdoc");
//        }
    }

}
