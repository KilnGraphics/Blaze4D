package me.hydos.blaze4d;

import java.io.FileOutputStream;
import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.IntBuffer;
import java.nio.LongBuffer;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.HashMap;
import java.util.Map;
import java.util.stream.Collectors;

import com.oroarmor.aftermath.Aftermath;
import com.oroarmor.aftermath.AftermathCallbackCreationHelper;
import com.oroarmor.aftermath.struct.GFSDK_Aftermath_GpuCrashDump_BaseInfo;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.system.MemoryUtil;

import net.minecraft.client.MinecraftClient;

public class AftermathHandler {
    private final Map<long[], String> SHADER_DEBUG_INFO = new HashMap<>();
    private final ShaderDatabase shaderDatabase = new ShaderDatabase();

    public static void initialize() {
        int result = Aftermath.enableGPUCrashDumps(
                Aftermath.AFTERMATH_API_VERSION,
                Aftermath.GPU_CRASH_DUMP_WATCHED_API_FLAGS_VULKAN,
                Aftermath.GPU_CRASH_DUMP_FEATURE_FLAGS_DEFER_DEBUG_INFO_CALLBACKS,
                AftermathHandler::onGpuCrashDump,
                AftermathHandler::onShaderDebugInfo,
                AftermathHandler::onGpuCrashDescription,
                new AftermathHandler()
        );
        if (result != 1) {
            Blaze4D.LOGGER.error("Failed to initialize Aftermath 0x%x", result);
        }
    }

    private static void onGpuCrashDump(long pGpuCrashDump, int gpuCrashDumpSize, long pUserData) {
        AftermathHandler handler = MemoryUtil.memGlobalRefToObject(pUserData);
        handler.onGpuCrashDump(pGpuCrashDump, gpuCrashDumpSize);
    }

    private static void onShaderDebugInfo(long pShaderDebugInfo, int shaderDebugInfoSize, long pUserData) {
        AftermathHandler handler = MemoryUtil.memGlobalRefToObject(pUserData);
        handler.onShaderDebugInfo(pShaderDebugInfo, shaderDebugInfoSize);
    }

    private static void onGpuCrashDescription(long addValueCallback, long pUserData) {
        AftermathHandler handler = MemoryUtil.memGlobalRefToObject(pUserData);
        handler.onGpuCrashDescription(addValueCallback);
    }

    private void onGpuCrashDescription(long addValueCallback) {
        Map<Integer, String> info = Map.of(Aftermath.GPU_CRASH_DUMP_DESCRIPTION_KEY_APPLICATION_NAME, "Blaze 4D",
                Aftermath.GPU_CRASH_DUMP_DESCRIPTION_KEY_APPLICATION_VERSION, "v1.0",
                Aftermath.GPU_CRASH_DUMP_DESCRIPTION_KEY_USER_DEFINED, "Gpu Crash Dump Blaze4D Info",
                Aftermath.GPU_CRASH_DUMP_DESCRIPTION_KEY_USER_DEFINED + 1, "Engine State: Rendering.",
                Aftermath.GPU_CRASH_DUMP_DESCRIPTION_KEY_USER_DEFINED + 2, "Current Screen: " + MinecraftClient.getInstance().currentScreen
        );
        info.forEach(AftermathCallbackCreationHelper.createAddGpuCrashDumpDescription(addValueCallback));
    }

    private void onShaderDebugInfo(long pShaderDebugInfo, int shaderDebugInfoSize) {
        try (MemoryStack stack = MemoryStack.stackPush()) {
            LongBuffer pIdentifier = stack.callocLong(2);
            Aftermath.getShaderDebugInfoIdentifier(
                    Aftermath.AFTERMATH_API_VERSION,
                    pShaderDebugInfo,
                    shaderDebugInfoSize,
                    pIdentifier);

            // Store information for decoding of GPU crash dumps with shader address mapping
            // from within the application.
            String data = MemoryUtil.memUTF8(pShaderDebugInfo, shaderDebugInfoSize);
            SHADER_DEBUG_INFO.put(pIdentifier.array(), data);

            // Write to file for later in-depth analysis of crash dumps with Nsight Graphics
            try {
                Files.writeString(Path.of(".", "shader-" + Arrays.stream(pIdentifier.array()).mapToObj(Long::toString).collect(Collectors.joining("-")) + ".nvdbg"), data);
            } catch (IOException e) {
                e.printStackTrace();
            }
        }
    }

    private void onGpuCrashDump(long pGpuCrashDump, int gpuCrashDumpSize) {
        try (MemoryStack stack = MemoryStack.stackPush()) {
            // Create a GPU crash dump decoder object for the GPU crash dump.
            LongBuffer pDecoder = stack.callocLong(1);
            Aftermath.createDecoder(
                    Aftermath.AFTERMATH_API_VERSION,
                    pGpuCrashDump,
                    gpuCrashDumpSize,
                    pDecoder
            );

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
                    Aftermath.GPU_CRASH_DUMP_DESCRIPTION_KEY_APPLICATION_NAME,
                    pApplicationNameLength);

            int size = pApplicationNameLength.get(0);
            ByteBuffer pApplicationName = stack.calloc(size);
            Aftermath.getDescription(
                    decoder,
                    Aftermath.GPU_CRASH_DUMP_DESCRIPTION_KEY_APPLICATION_NAME,
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
                    Aftermath.GPU_CRASH_DUMP_DECODER_FLAGS_ALL_INFO,
                    Aftermath.GPU_CRASH_DUMP_FORMATTER_FLAGS_UTF8_OUTPUT,
                    (pIdentifier, setShaderDebugInfo, pUserData) -> {
                        // Search the list of shader debug information blobs received earlier.
                        String i_debugInfo = SHADER_DEBUG_INFO.get(pIdentifier);
                        if (i_debugInfo != null) {
                            // Let the GPU crash dump decoder know about the shader debug information that was found.
                            setShaderDebugInfo.apply(MemoryUtil.memUTF8(i_debugInfo), i_debugInfo.length());
                        }
                    },
                    (shaderHash, setShaderBinary, pUserData) -> {
                        // Find shader binary data for the shader hash in the shader database.
                        ByteBuffer shaderBinary = shaderDatabase.findShaderBinary(shaderHash);
                        if (shaderBinary != null) {
                            // Let the GPU crash dump decoder know about the shader data that was found.
                            setShaderBinary.apply(shaderBinary, shaderBinary.capacity());
                        }
                    },
                    (pShaderInstructionsHash, setShaderBinary, pUserData) -> {
                    },
                    (shaderDebugName, setShaderBinary, pUserData) -> {
                        ByteBuffer shaderBinary = shaderDatabase.findShaderBinaryWithDebugData(shaderDebugName);
                        if (shaderBinary != null) {
                            // Let the GPU crash dump decoder know about the shader data that was found.
                            setShaderBinary.apply(shaderBinary, shaderBinary.capacity());
                        }
                    },
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
            jsonDumpFile.write(MemoryUtil.memUTF8Safe(pJsonBuffer).getBytes(StandardCharsets.UTF_8));

            // Destroy the GPU crash dump decoder object.
            Aftermath.destroyDecoder(decoder);
        } catch (Exception e) {
            throw new RuntimeException("Failed to write Gpu crash dump to file", e);
        }
    }

    private static class ShaderDatabase {
        public ByteBuffer findShaderBinary(long shaderHash) {
            return null;
        }

        public ByteBuffer findShaderBinaryWithDebugData(String shaderDebugName) {
            return null;
        }
    }
}
