package graphics.kiln.blaze4d;

import net.fabricmc.api.ClientModInitializer;
import jdk.incubator.foreign.*;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;
import org.apache.logging.log4j.message.StringFormatterMessageFactory;

import java.lang.invoke.MethodHandle;
import java.lang.invoke.MethodType;
import java.util.Optional;

public class Blaze4D implements ClientModInitializer {

//    public static final Logger LOGGER = LogManager.getLogger("Blaze4D", new StringFormatterMessageFactory());
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
        System.load("/home/lrai/Documents/Dev/b4d_core/target/debug/libb4d_core.so");
        Optional<MemoryAddress> result = SymbolLookup.loaderLookup().lookup("b4d_core_init");

        if (result.isPresent()) {
            MethodHandle init = CLinker.getInstance().downcallHandle(
                    result.get(),
                    MethodType.methodType(Void.class),
                    FunctionDescriptor.ofVoid()
            );
            try {
                init.invoke();
            } catch (Throwable e) {
                throw new RuntimeException(e);
            }
        } else {
            LogManager.getLogger().fatal("Failed to find init");
        }


//        if (RENDERDOC_ENABLED) {
//            System.loadLibrary("renderdoc");
//        }
    }
}
