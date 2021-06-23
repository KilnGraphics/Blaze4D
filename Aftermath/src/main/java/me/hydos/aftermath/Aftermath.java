package me.hydos.aftermath;

import me.hydos.aftermath.callback.GPUCrashDumpCallbackI;
import me.hydos.aftermath.callback.GpuCrashDumpDescriptionCallbackI;
import me.hydos.aftermath.callback.ShaderDebugInfoCallbackI;
import me.hydos.aftermath.struct.GFSDK_Aftermath_GpuCrashDump_BaseInfo;
import org.lwjgl.system.*;

import java.nio.ByteBuffer;
import java.nio.IntBuffer;
import java.nio.LongBuffer;

public class Aftermath {
    private static final SharedLibrary AFTERMATH = Library.loadNative(Aftermath.class,
            "me.hydos.aftermath",
            System.getProperty("os.name").toLowerCase().contains("windows") ? "GFSDK_Aftermath_Lib.x64" : "",
            false);

    private static final long GFSDK_Aftermath_EnableGpuCrashDumps;
    private static final long GFSDK_Aftermath_DisableGpuCrashDumps;
    public static final long GFSDK_Aftermath_GpuCrashDump_CreateDecoder;
    public static final long GFSDK_Aftermath_GpuCrashDump_DestroyDecoder;
    public static final long GFSDK_Aftermath_GpuCrashDump_GetBaseInfo;
    public static final long GFSDK_Aftermath_GpuCrashDump_GetDescriptionSize;
    public static final long GFSDK_Aftermath_GpuCrashDump_GetDescription;
    public static final long GFSDK_Aftermath_GpuCrashDump_GetDeviceInfo;
    public static final long GFSDK_Aftermath_GpuCrashDump_GetSystemInfo;
    public static final long GFSDK_Aftermath_GpuCrashDump_GetGpuInfoCount;
    public static final long GFSDK_Aftermath_GpuCrashDump_GetGpuInfo;
    public static final long GFSDK_Aftermath_GpuCrashDump_GetPageFaultInfo;
    public static final long GFSDK_Aftermath_GpuCrashDump_GetActiveShadersInfoCount;
    public static final long GFSDK_Aftermath_GpuCrashDump_GetActiveShadersInfo;
    public static final long GFSDK_Aftermath_GpuCrashDump_GetEventMarkersInfoCount;
    public static final long GFSDK_Aftermath_GpuCrashDump_GetEventMarkersInfo;
    public static final long GFSDK_Aftermath_GpuCrashDump_GenerateJSON;
    public static final long GFSDK_Aftermath_GpuCrashDump_GetJSON;
    public static final long GFSDK_Aftermath_GetShaderDebugInfoIdentifier;

    static {
        GFSDK_Aftermath_DisableGpuCrashDumps = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_DisableGpuCrashDumps");
        GFSDK_Aftermath_EnableGpuCrashDumps = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_EnableGpuCrashDumps");
        GFSDK_Aftermath_GpuCrashDump_CreateDecoder = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_GpuCrashDump_CreateDecoder");
        GFSDK_Aftermath_GpuCrashDump_DestroyDecoder = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_GpuCrashDump_DestroyDecoder");
        GFSDK_Aftermath_GpuCrashDump_GetBaseInfo = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_GpuCrashDump_GetBaseInfo");
        GFSDK_Aftermath_GpuCrashDump_GetDescriptionSize = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_GpuCrashDump_GetDescriptionSize");
        GFSDK_Aftermath_GpuCrashDump_GetDescription = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_GpuCrashDump_GetDescription");
        GFSDK_Aftermath_GpuCrashDump_GetDeviceInfo = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_GpuCrashDump_GetDeviceInfo");
        GFSDK_Aftermath_GpuCrashDump_GetSystemInfo = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_GpuCrashDump_GetSystemInfo");
        GFSDK_Aftermath_GpuCrashDump_GetGpuInfoCount = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_GpuCrashDump_GetGpuInfoCount");
        GFSDK_Aftermath_GpuCrashDump_GetGpuInfo = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_GpuCrashDump_GetGpuInfo");
        GFSDK_Aftermath_GpuCrashDump_GetPageFaultInfo = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_GpuCrashDump_GetPageFaultInfo");
        GFSDK_Aftermath_GpuCrashDump_GetActiveShadersInfoCount = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_GpuCrashDump_GetActiveShadersInfoCount");
        GFSDK_Aftermath_GpuCrashDump_GetActiveShadersInfo = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_GpuCrashDump_GetActiveShadersInfo");
        GFSDK_Aftermath_GpuCrashDump_GetEventMarkersInfoCount = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_GpuCrashDump_GetEventMarkersInfoCount");
        GFSDK_Aftermath_GpuCrashDump_GetEventMarkersInfo = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_GpuCrashDump_GetEventMarkersInfo");
        GFSDK_Aftermath_GpuCrashDump_GenerateJSON = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_GpuCrashDump_GenerateJSON");
        GFSDK_Aftermath_GpuCrashDump_GetJSON = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_GpuCrashDump_GetJSON");
        GFSDK_Aftermath_GetShaderDebugInfoIdentifier = APIUtil.apiGetFunctionAddress(AFTERMATH, "GFSDK_Aftermath_GetShaderDebugInfoIdentifier");
    }

    public static int disableGPUCrashDumps() {
        return (int) JNI.invokeJ(GFSDK_Aftermath_DisableGpuCrashDumps);
    }


    public static int enableGPUCrashDumps(long version, int watchedAPIs, int flags,
                                          GPUCrashDumpCallbackI gpuCrashDumpCb, ShaderDebugInfoCallbackI shaderDebugInfCb,
                                          GpuCrashDumpDescriptionCallbackI gpuCrashDumpDescriptionCb, Object userData) {
        return (int) JNI.callPPJPPPPP(
                version,
                watchedAPIs,
                flags,
                gpuCrashDumpCb.address(),
                shaderDebugInfCb.address(),
                gpuCrashDumpDescriptionCb.address(),
                MemoryUtil.NULL,
                GFSDK_Aftermath_EnableGpuCrashDumps
        );
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
}
