package me.hydos.blaze4d;

import me.hydos.aftermath.AddGPUCrashDumpDescriptionCallback;
import me.hydos.aftermath.Aftermath;
import me.hydos.aftermath.struct.GFSDK_Aftermath_GpuCrashDump_BaseInfo;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.render.io.Window;
import net.fabricmc.api.ModInitializer;
import net.minecraft.client.MinecraftClient;
import org.apache.logging.log4j.Level;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;
import org.apache.logging.log4j.message.StringFormatterMessageFactory;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.system.MemoryUtil;

import java.io.FileOutputStream;
import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.IntBuffer;
import java.nio.LongBuffer;
import java.nio.charset.StandardCharsets;
import java.util.Map;

public class Blaze4D implements ModInitializer {
    public static final Logger LOGGER = LogManager.getLogger("Iodine", new StringFormatterMessageFactory());

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

        Aftermath.enableGPUCrashDumps(
                0x000020b,
                0x2,
                0x1,
                (pGpuCrashDump, gpuCrashDumpSize, pUserData) -> {
                    try {
                        writeGpuCrashDumpToFile(pGpuCrashDump, gpuCrashDumpSize);
                    } catch (IOException e) {
                        throw new IOException("Failed to write Gpu crash dump to file", e);
                    }
                },
                (pShaderDebugInfo, shaderDebugInfoSize, pUserData) -> {},
                (addValue, pUserData) -> {
                    Map<Integer, String> info = Map.of(0x00000001, "Blaze 4D",
                            0x00000002, "v1.0",
                            0x00010000, "Gpu Crash Dump Blaze4D Info",
                            0x00010000 + 1, "Engine State: Rendering.",
                            0x00010000 + 2, "Current Screen: " + MinecraftClient.getInstance().currentScreen.getTitle().asString()
                    );

                    info.forEach(AddGPUCrashDumpDescriptionCallback.invoke(addValue));
                },
                new Object()
        );
    }

    private static void writeGpuCrashDumpToFile(long pGpuCrashDump, int gpuCrashDumpSize) throws IOException {
        try (MemoryStack stack = MemoryStack.stackPush()) {
            // Create a GPU crash dump decoder object for the GPU crash dump.
            LongBuffer pDecoder = stack.callocLong(1);
            Aftermath.createDecoder(
                    0x000020b,
                    pGpuCrashDump,
                    gpuCrashDumpSize,
                    pDecoder);

            long decoder = pDecoder.get(0);

            // Use the decoder object to read basic information, like application
            // name, PID, etc. from the GPU crash dump.
            GFSDK_Aftermath_GpuCrashDump_BaseInfo baseInfo = GFSDK_Aftermath_GpuCrashDump_BaseInfo.create();
            Aftermath.getBaseInfo(decoder, baseInfo);

            // Use the decoder object to query the application name that was set
            // in the GPU crash dump description.
            IntBuffer pApplicationNameLength = stack.callocInt(1);
            Aftermath.getDescriptionSize(
                    decoder,
                    0x00000001,
                    pApplicationNameLength);

            int size = pApplicationNameLength.get(0);
            ByteBuffer pApplicationName = stack.calloc(size);
            Aftermath.getDescription(
                    decoder,
                    0x00000001,
                    size,
                    pApplicationName);

            // Create a unique file name for writing the crash dump data to a file.
            // Note: due to an Nsight Aftermath bug (will be fixed in an upcoming
            // driver release) we may see redundant crash dumps. As a workaround,
            // attach a unique count to each generated file name.
            String applicationName = MemoryUtil.memUTF8(pApplicationName, size - 1);
            String baseFileName = applicationName + "-" + baseInfo.pid();

            // Write the the crash dump data to a file using the .nv-gpudmp extension
            // registered with Nsight Graphics.
            String crashDumpFileName = baseFileName + ".nv-gpudmp";
            FileOutputStream dumpFile = new FileOutputStream(crashDumpFileName);
            dumpFile.write(MemoryUtil.memUTF8Safe(pGpuCrashDump, gpuCrashDumpSize).getBytes(StandardCharsets.UTF_8));

            // Decode the crash dump to a JSON string.
            // Step 1: Generate the JSON and get the size.
            IntBuffer pJsonSize = stack.callocInt(1);
            Aftermath.generateJson(
                    decoder,
                    0xFFF,
                    0x0,
                    MemoryUtil.NULL, // (pIdentifier, setShaderDebugInfo, pUserData) -> {},
                    MemoryUtil.NULL, // (pShaderHash, setShaderBinary , pUserData) -> {},
                    MemoryUtil.NULL,
                    MemoryUtil.NULL,
                    new Object(),
                    pJsonSize);
            // Step 2: Allocate a buffer and fetch the generated JSON.
            ByteBuffer pJsonBuffer = stack.malloc(pJsonSize.get(0));
            Aftermath.getJson(
                    decoder,
                    pJsonSize.get(0),
                    pJsonBuffer);

            // Write the the crash dump data as JSON to a file.
            String jsonFileName = crashDumpFileName + ".json";
            FileOutputStream jsonDumpFile = new FileOutputStream(jsonFileName);
            jsonDumpFile.write(MemoryUtil.memUTF8(pJsonBuffer).getBytes(StandardCharsets.UTF_8));

            // Destroy the GPU crash dump decoder object.
            Aftermath.destroyDecoder(decoder);
        }
    }
}
