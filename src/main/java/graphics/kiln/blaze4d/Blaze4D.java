package graphics.kiln.blaze4d;

import graphics.kiln.blaze4d.api.Blaze4DCore;
import jdk.incubator.foreign.*;
import net.fabricmc.api.ClientModInitializer;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;
import org.apache.logging.log4j.message.StringFormatterMessageFactory;
import org.lwjgl.glfw.GLFW;
import org.lwjgl.system.APIUtil;

import java.lang.invoke.MethodHandle;
import java.lang.invoke.MethodHandles;
import java.lang.invoke.MethodType;
import java.nio.charset.StandardCharsets;

public class Blaze4D implements ClientModInitializer {

    public static final Logger LOGGER = LogManager.getLogger("Blaze4D", new StringFormatterMessageFactory());
    private static final Logger NATIVE_LOGGER = LogManager.getLogger("Blaze4DNative", new StringFormatterMessageFactory());

    public static Blaze4DCore core;
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

        try {
            MethodHandle logHandle = MethodHandles.lookup().findStatic(Blaze4D.class, "b4dLogFn",
                    MethodType.methodType(Void.TYPE, MemoryAddress.class, MemoryAddress.class, Integer.TYPE, Integer.TYPE, Integer.TYPE));
            NativeSymbol nativeSymbol = Blaze4DNatives.linker.upcallStub(
                    logHandle,
                    FunctionDescriptor.ofVoid(ValueLayout.ADDRESS, ValueLayout.ADDRESS, ValueLayout.JAVA_INT, ValueLayout.JAVA_INT, ValueLayout.JAVA_INT),
                    ResourceScope.globalScope()
            );
            Blaze4DNatives.b4dInitExternalLogger(nativeSymbol);
        } catch (Exception ex) {
            LOGGER.error("Failed to initialize b4d external logger", ex);
            throw new RuntimeException(ex);
        }


        Blaze4DNatives.b4dPreInitGlfw(MemoryAddress.ofLong(APIUtil.apiGetFunctionAddress(GLFW.getLibrary(), "glfwInitVulkanLoader")));

//        if (RENDERDOC_ENABLED) {
            System.load("C:\\Program Files\\RenderDoc\\renderdoc.dll");
//        }
    }

    private static void b4dLogFn(MemoryAddress targetPtr, MemoryAddress msgPtr, int targetLen, int msgLen, int level) {
        try (ResourceScope scope = ResourceScope.newConfinedScope()) {
            MemorySegment target = MemorySegment.ofAddress(targetPtr, targetLen, scope);
            MemorySegment message = MemorySegment.ofAddress(msgPtr, msgLen, scope);

            byte[] targetData = target.toArray(ValueLayout.JAVA_BYTE);
            byte[] messageData = message.toArray(ValueLayout.JAVA_BYTE);

            String targetString = new String(targetData, StandardCharsets.UTF_8);
            String messageString = new String(messageData, StandardCharsets.UTF_8);

            switch (level) {
                case 0 -> NATIVE_LOGGER.trace(messageString);
                case 1 -> NATIVE_LOGGER.debug(messageString);
                case 2 -> NATIVE_LOGGER.info(messageString);
                case 3 -> NATIVE_LOGGER.warn(messageString);
                case 4 -> NATIVE_LOGGER.error(messageString);
                default -> LOGGER.error("Received invalid log level from b4d native: " + level);
            }
        } catch (Throwable e) {
            LOGGER.error("Failed to log native message", e);
        }
    }
}
