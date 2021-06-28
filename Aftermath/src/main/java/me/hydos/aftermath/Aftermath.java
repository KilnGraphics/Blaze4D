package me.hydos.aftermath;

import me.hydos.aftermath.callback.*;
import me.hydos.aftermath.struct.GFSDK_Aftermath_GpuCrashDump_BaseInfo;
import org.lwjgl.system.*;
import org.lwjgl.system.jni.JNINativeInterface;

import java.nio.ByteBuffer;
import java.nio.IntBuffer;
import java.nio.LongBuffer;
import java.util.function.BiConsumer;
import java.util.function.BiFunction;

public class Aftermath {
    private static final SharedLibrary AFTERMATH = Library.loadNative(Aftermath.class,
            "me.hydos.aftermath",
            System.getProperty("os.name").toLowerCase().contains("windows") ? "WinAftermath.dll" : "LinuxAftermath",
            false);

    private static final long GFSDK_Aftermath_EnableGpuCrashDumps = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_DisableGpuCrashDumps");
    private static final long GFSDK_Aftermath_DisableGpuCrashDumps = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_EnableGpuCrashDumps");
    private static final long GFSDK_Aftermath_GpuCrashDump_CreateDecoder = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_GpuCrashDump_CreateDecoder");
    private static final long GFSDK_Aftermath_GpuCrashDump_DestroyDecoder = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_GpuCrashDump_DestroyDecoder");
    private static final long GFSDK_Aftermath_GpuCrashDump_GetBaseInfo = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_GpuCrashDump_GetBaseInfo");
    private static final long GFSDK_Aftermath_GpuCrashDump_GetDescriptionSize = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_GpuCrashDump_GetDescriptionSize");
    private static final long GFSDK_Aftermath_GpuCrashDump_GetDescription = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_GpuCrashDump_GetDescription");
    private static final long GFSDK_Aftermath_GpuCrashDump_GetDeviceInfo = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_GpuCrashDump_GetDeviceInfo");
    private static final long GFSDK_Aftermath_GpuCrashDump_GetSystemInfo = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_GpuCrashDump_GetSystemInfo");
    private static final long GFSDK_Aftermath_GpuCrashDump_GetGpuInfoCount = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_GpuCrashDump_GetGpuInfoCount");
    private static final long GFSDK_Aftermath_GpuCrashDump_GetGpuInfo = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_GpuCrashDump_GetGpuInfo");
    private static final long GFSDK_Aftermath_GpuCrashDump_GetPageFaultInfo = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_GpuCrashDump_GetPageFaultInfo");
    private static final long GFSDK_Aftermath_GpuCrashDump_GetActiveShadersInfoCount = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_GpuCrashDump_GetActiveShadersInfoCount");
    private static final long GFSDK_Aftermath_GpuCrashDump_GetActiveShadersInfo = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_GpuCrashDump_GetActiveShadersInfo");
    private static final long GFSDK_Aftermath_GpuCrashDump_GetEventMarkersInfoCount = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_GpuCrashDump_GetEventMarkersInfoCount");
    private static final long GFSDK_Aftermath_GpuCrashDump_GetEventMarkersInfo = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_GpuCrashDump_GetEventMarkersInfo");
    private static final long GFSDK_Aftermath_GpuCrashDump_GenerateJSON = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_GpuCrashDump_GenerateJSON");
    private static final long GFSDK_Aftermath_GpuCrashDump_GetJSON = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_GpuCrashDump_GetJSON");
    private static final long GFSDK_Aftermath_GetShaderDebugInfoIdentifier = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_GetShaderDebugInfoIdentifier");

    /**
     * Aftermath API Versions
     */
    public static final int AFTERMATH_API_VERSION = 0x000020b;


    /**
     * Aftermath GPU Crash Dump Description Keys
     */
    public static final int GPU_CRASH_DUMP_DESCRIPTION_KEY_APPLICATION_NAME = 0x00000001;
    public static final int GPU_CRASH_DUMP_DESCRIPTION_KEY_APPLICATION_VERSION = 0x00000002;
    public static final int GPU_CRASH_DUMP_DESCRIPTION_KEY_USER_DEFINED = 0x00010000;

    /**
     * Aftermath watched APIs
     */
    public static final int GPU_CRASH_DUMP_WATCHED_API_FLAGS_NONE = 0x0;
    public static final int GPU_CRASH_DUMP_WATCHED_API_FLAGS_DX = 0x1;
    public static final int GPU_CRASH_DUMP_WATCHED_API_FLAGS_VULKAN = 0x2;

    /**
     * Aftermath feature flags
     */
    public static final int GPU_CRASH_DUMP_FEATURE_FLAGS_DEFAULT = 0x0;
    public static final int GPU_CRASH_DUMP_FEATURE_FLAGS_DEFER_DEBUG_INFO_CALLBACKS = 0x1;

    /**
     * Aftermath decoder flags
     */
    public static final int GPU_CRASH_DUMP_DECODER_FLAGS_BASE_INFO = 0x1;
    public static final int GPU_CRASH_DUMP_DECODER_FLAGS_DEVICE_INFO = 0x2;
    public static final int GPU_CRASH_DUMP_DECODER_FLAGS_OS_INFO = 0x4;
    public static final int GPU_CRASH_DUMP_DECODER_FLAGS_DRIVER_INFO = 0x8;
    public static final int GPU_CRASH_DUMP_DECODER_FLAGS_GPU_INFO = 0x10;
    public static final int GPU_CRASH_DUMP_DECODER_FLAGS_FAULT_INFO = 0x20;
    public static final int GPU_CRASH_DUMP_DECODER_FLAGS_SHADER_INFO = 0x40;
    public static final int GPU_CRASH_DUMP_DECODER_FLAGS_STATE_INFO = 0x80;
    public static final int GPU_CRASH_DUMP_DECODER_FLAGS_MAPPING_INFO = 0x100;
    public static final int GPU_CRASH_DUMP_DECODER_FLAGS_MARKER_INFO = 0x200;
    public static final int GPU_CRASH_DUMP_DECODER_FLAGS_STACK_INFO = 0x400;
    public static final int GPU_CRASH_DUMP_DECODER_FLAGS_DESCRIPTION_INFO = 0x800;
    public static final int GPU_CRASH_DUMP_DECODER_FLAGS_ALL_INFO = 0xFFF;

    /**
     * Aftermath formatter flags
     */
    public static final int GPU_CRASH_DUMP_FORMATTER_FLAGS_NONE = 0x0;
    public static final int GPU_CRASH_DUMP_FORMATTER_FLAGS_CONDENSED_OUTPUT = 0x1;
    public static final int GPU_CRASH_DUMP_FORMATTER_FLAGS_UTF8_OUTPUT = 0x2;

    public static int disableGPUCrashDumps() {
        return (int) JNI.invokeJ(GFSDK_Aftermath_DisableGpuCrashDumps);
    }

    public static int enableGPUCrashDumps(long version, int watchedAPIs, int flags, GPUCrashDumpCallbackI gpuCrashDumpCb, ShaderDebugInfoCallbackI shaderDebugInfCb, GpuCrashDumpDescriptionCallbackI gpuCrashDumpDescriptionCb, Object pUserData) {
        return (int) JNI.callPPJPPPPP(version, watchedAPIs, flags, gpuCrashDumpCb.address(), shaderDebugInfCb.address(), gpuCrashDumpDescriptionCb.address(), JNINativeInterface.NewGlobalRef(pUserData), GFSDK_Aftermath_EnableGpuCrashDumps);
    }

    public static int createDecoder(int apiVersion, long pGpuCrashDump, int gpuCrashDumpSize, LongBuffer pDecoder) {
        return JNI.invokePPPPI(apiVersion, pGpuCrashDump, gpuCrashDumpSize, MemoryUtil.memAddress(pDecoder), GFSDK_Aftermath_GpuCrashDump_CreateDecoder);
    }

    public static int getBaseInfo(long decoder, GFSDK_Aftermath_GpuCrashDump_BaseInfo pBaseInfo) {
        return JNI.invokePPI(decoder, pBaseInfo.address(), GFSDK_Aftermath_GpuCrashDump_GetBaseInfo);
    }

    public static int getDescriptionSize(long decoder, int key, IntBuffer pApplicationNameLength) {
        return JNI.invokePPPI(decoder, key, MemoryUtil.memAddress(pApplicationNameLength), GFSDK_Aftermath_GpuCrashDump_GetDescriptionSize);
    }

    public static int getDescription(long decoder, int key, int size, ByteBuffer pApplicationName) {
        return JNI.invokePPPPI(decoder, key, size, MemoryUtil.memAddress(pApplicationName), GFSDK_Aftermath_GpuCrashDump_GetDescription);
    }

    public static int generateJson(long decoder, int decoderFlags, int formatFlags, ShaderDebugInfoLookupCallbackI shaderDebugLookupCb, ShaderLookupCallbackI shaderLookupCb, ShaderInstructionsLookupCallbackI shaderInstructionsLookupCb, ShaderSourceDebugInfoLookupCallbackI shaderSourceDebugInfoLookupCb, Object pUserData, IntBuffer pJsonSize) {
        return JNI.callJPPPPPPPPI(decoder, decoderFlags, formatFlags, shaderDebugLookupCb.address(), shaderLookupCb.address(), shaderInstructionsLookupCb.address(), shaderSourceDebugInfoLookupCb.address(), JNINativeInterface.NewGlobalRef(pUserData), MemoryUtil.memAddressSafe(pJsonSize), GFSDK_Aftermath_GpuCrashDump_GenerateJSON);
    }

    public static int getJson(long decoder, int jsonSize, ByteBuffer pJsonBuffer) {
        return JNI.callPPPI(decoder, jsonSize, MemoryUtil.memAddress(pJsonBuffer), GFSDK_Aftermath_GpuCrashDump_GetJSON);
    }

    public static int destroyDecoder(long decoder) {
        return JNI.callPI(decoder, GFSDK_Aftermath_GpuCrashDump_DestroyDecoder);
    }

    public static int getShaderDebugInfoIdentifier(int apiVersion, long pShaderDebugInfo, int shaderDebugInfoSize, LongBuffer pIdentifier) {
        return JNI.callPPPPI(apiVersion, pShaderDebugInfo, shaderDebugInfoSize, MemoryUtil.memAddress(pIdentifier), GFSDK_Aftermath_GetShaderDebugInfoIdentifier);
    }
}
