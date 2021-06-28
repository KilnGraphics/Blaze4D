package me.hydos.aftermath;

import org.lwjgl.system.JNI;
import org.lwjgl.system.MemoryUtil;

import java.nio.ByteBuffer;
import java.util.function.BiConsumer;
import java.util.function.BiFunction;

public abstract class AftermathCallbackCreationHelper {
   public static BiConsumer<Integer, String> createAddGpuCrashDumpDescription(long address) {
       return (integer, s) -> JNI.invokePPV(integer, MemoryUtil.memAddress(MemoryUtil.memUTF8(s, true)), address);
   }

    public static BiFunction<ByteBuffer, Integer, Integer> createSetShaderDebugInfo(long setShaderDebugInfo) {
        return (bytes, length) -> JNI.callPPI(MemoryUtil.memAddress(bytes), length, setShaderDebugInfo);
    }
}
