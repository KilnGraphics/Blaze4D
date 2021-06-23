package me.hydos.aftermath.struct;

import org.jetbrains.annotations.Nullable;
import org.lwjgl.BufferUtils;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.system.MemoryUtil;
import org.lwjgl.system.NativeResource;
import org.lwjgl.system.Struct;

import java.nio.ByteBuffer;

import static org.lwjgl.system.MemoryStack.stackGet;
import static org.lwjgl.system.MemoryUtil.*;
import static org.lwjgl.system.MemoryUtil.NULL;

public class GFSDK_Aftermath_GpuCrashDump_BaseInfo extends Struct implements NativeResource {

    public static final int SIZEOF;
    public static final int ALIGNOF;

    public static final int APPLICATION_NAME, CREATION_DATE, PID, GRAPHICS_API;

    static {
        Layout layout = __struct(
                __member(128),
                __member(128),
                __member(4),
                __member(4)
        );

        SIZEOF = layout.getSize();
        ALIGNOF = layout.getAlignment();

        APPLICATION_NAME = layout.offsetof(0);
        CREATION_DATE = layout.offsetof(1);
        PID = layout.offsetof(2);
        GRAPHICS_API = layout.offsetof(3);
    }

    protected GFSDK_Aftermath_GpuCrashDump_BaseInfo(ByteBuffer container) {
        super(MemoryUtil.memAddress(container), container);
    }

    @Override
    public int sizeof() {
        return SIZEOF;
    }

    public String applicationName() {
        return nApplicationName(address());
    }

    public static String nApplicationName(long address) {
        return MemoryUtil.memUTF8(address + APPLICATION_NAME, 128);
    }

    public String creationDate() {
        return nCreationDate(address());
    }

    public static String nCreationDate(long address) {
        return MemoryUtil.memUTF8(address + CREATION_DATE, 128);
    }

    public int pid() {
        return npid(address());
    }

    public static int npid(long address) {
        return UNSAFE.getInt(null, address + PID);
    }

    public int graphicsApi() {
        return nGraphicsApi(address());
    }

    public static int nGraphicsApi(long address) {
        return UNSAFE.getInt(null, address + GRAPHICS_API);
    }

    public static GFSDK_Aftermath_GpuCrashDump_BaseInfo malloc() {
        return wrap(GFSDK_Aftermath_GpuCrashDump_BaseInfo.class, nmemAllocChecked(SIZEOF));
    }

    public static GFSDK_Aftermath_GpuCrashDump_BaseInfo calloc() {
        return wrap(GFSDK_Aftermath_GpuCrashDump_BaseInfo.class, nmemCallocChecked(1, SIZEOF));
    }

    public static GFSDK_Aftermath_GpuCrashDump_BaseInfo create() {
        ByteBuffer container = BufferUtils.createByteBuffer(SIZEOF);
        return wrap(GFSDK_Aftermath_GpuCrashDump_BaseInfo.class, memAddress(container), container);
    }

    public static GFSDK_Aftermath_GpuCrashDump_BaseInfo create(long address) {
        return wrap(GFSDK_Aftermath_GpuCrashDump_BaseInfo.class, address);
    }

    @Nullable
    public static GFSDK_Aftermath_GpuCrashDump_BaseInfo createSafe(long address) {
        return address == NULL ? null : wrap(GFSDK_Aftermath_GpuCrashDump_BaseInfo.class, address);
    }

    public static GFSDK_Aftermath_GpuCrashDump_BaseInfo mallocStack() {
        return mallocStack(stackGet());
    }

    public static GFSDK_Aftermath_GpuCrashDump_BaseInfo callocStack() {
        return callocStack(stackGet());
    }

    public static GFSDK_Aftermath_GpuCrashDump_BaseInfo mallocStack(MemoryStack stack) {
        return wrap(GFSDK_Aftermath_GpuCrashDump_BaseInfo.class, stack.nmalloc(ALIGNOF, SIZEOF));
    }

    public static GFSDK_Aftermath_GpuCrashDump_BaseInfo callocStack(MemoryStack stack) {
        return wrap(GFSDK_Aftermath_GpuCrashDump_BaseInfo.class, stack.ncalloc(ALIGNOF, 1, SIZEOF));
    }


}
