package me.hydos.aftermath;

import org.jetbrains.annotations.Nullable;
import org.lwjgl.system.Callback;
import org.lwjgl.system.JNI;
import org.lwjgl.system.MemoryUtil;

import java.util.function.BiConsumer;

import static org.lwjgl.system.MemoryUtil.NULL;


public abstract class AddGPUCrashDumpDescriptionCallback {
   public static BiConsumer<Integer, String> invoke(long address) {
       return (integer, s) -> JNI.invokePPV(integer, MemoryUtil.memAddress(MemoryUtil.memUTF8(s, true)), address);
   }
}
