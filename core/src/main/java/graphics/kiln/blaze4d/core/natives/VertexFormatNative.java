package graphics.kiln.blaze4d.core.natives;

import graphics.kiln.blaze4d.core.Blaze4DCore;
import jdk.incubator.foreign.*;

import java.lang.invoke.VarHandle;

public class VertexFormatNative implements AutoCloseable {
    public static final MemoryLayout LAYOUT;

    public static final MemoryLayout.PathElement STRIDE_PATH;
    public static final MemoryLayout.PathElement POSITION_OFFSET_PATH;
    public static final MemoryLayout.PathElement POSITION_FORMAT_PATH;
    public static final MemoryLayout.PathElement NORMAL_OFFSET_PATH;
    public static final MemoryLayout.PathElement NORMAL_FORMAT_PATH;
    public static final MemoryLayout.PathElement COLOR_OFFSET_PATH;
    public static final MemoryLayout.PathElement COLOR_FORMAT_PATH;
    public static final MemoryLayout.PathElement UV0_OFFSET_PATH;
    public static final MemoryLayout.PathElement UV0_FORMAT_PATH;
    public static final MemoryLayout.PathElement UV1_OFFSET_PATH;
    public static final MemoryLayout.PathElement UV1_FORMAT_PATH;
    public static final MemoryLayout.PathElement UV2_OFFSET_PATH;
    public static final MemoryLayout.PathElement UV2_FORMAT_PATH;
    public static final MemoryLayout.PathElement HAS_NORMAL_PATH;
    public static final MemoryLayout.PathElement HAS_COLOR_PATH;
    public static final MemoryLayout.PathElement HAS_UV0_PATH;
    public static final MemoryLayout.PathElement HAS_UV1_PATH;
    public static final MemoryLayout.PathElement HAS_UV2_PATH;

    public static final VarHandle STRIDE_HANDLE;
    public static final VarHandle POSITION_OFFSET_HANDLE;
    public static final VarHandle POSITION_FORMAT_HANDLE;
    public static final VarHandle NORMAL_OFFSET_HANDLE;
    public static final VarHandle NORMAL_FORMAT_HANDLE;
    public static final VarHandle COLOR_OFFSET_HANDLE;
    public static final VarHandle COLOR_FORMAT_HANDLE;
    public static final VarHandle UV0_OFFSET_HANDLE;
    public static final VarHandle UV0_FORMAT_HANDLE;
    public static final VarHandle UV1_OFFSET_HANDLE;
    public static final VarHandle UV1_FORMAT_HANDLE;
    public static final VarHandle UV2_OFFSET_HANDLE;
    public static final VarHandle UV2_FORMAT_HANDLE;
    public static final VarHandle HAS_NORMAL_HANDLE;
    public static final VarHandle HAS_COLOR_HANDLE;
    public static final VarHandle HAS_UV0_HANDLE;
    public static final VarHandle HAS_UV1_HANDLE;
    public static final VarHandle HAS_UV2_HANDLE;

    static {
        LAYOUT = MemoryLayout.structLayout(
                ValueLayout.JAVA_INT.withName("stride"),
                ValueLayout.JAVA_INT.withName("position_offset"),
                ValueLayout.JAVA_INT.withName("position_format"),
                ValueLayout.JAVA_INT.withName("normal_offset"),
                ValueLayout.JAVA_INT.withName("normal_format"),
                ValueLayout.JAVA_INT.withName("color_offset"),
                ValueLayout.JAVA_INT.withName("color_format"),
                ValueLayout.JAVA_INT.withName("uv0_offset"),
                ValueLayout.JAVA_INT.withName("uv0_format"),
                ValueLayout.JAVA_INT.withName("uv1_offset"),
                ValueLayout.JAVA_INT.withName("uv1_format"),
                ValueLayout.JAVA_INT.withName("uv2_offset"),
                ValueLayout.JAVA_INT.withName("uv2_format"),
                ValueLayout.JAVA_BOOLEAN.withName("has_normal"),
                ValueLayout.JAVA_BOOLEAN.withName("has_color"),
                ValueLayout.JAVA_BOOLEAN.withName("has_uv0"),
                ValueLayout.JAVA_BOOLEAN.withName("has_uv1"),
                ValueLayout.JAVA_BOOLEAN.withName("has_uv2")
        );

        STRIDE_PATH = MemoryLayout.PathElement.groupElement("stride");
        POSITION_OFFSET_PATH = MemoryLayout.PathElement.groupElement("position_offset");
        POSITION_FORMAT_PATH = MemoryLayout.PathElement.groupElement("position_format");
        NORMAL_OFFSET_PATH = MemoryLayout.PathElement.groupElement("normal_offset");
        NORMAL_FORMAT_PATH = MemoryLayout.PathElement.groupElement("normal_format");
        COLOR_OFFSET_PATH = MemoryLayout.PathElement.groupElement("color_offset");
        COLOR_FORMAT_PATH = MemoryLayout.PathElement.groupElement("color_format");
        UV0_OFFSET_PATH = MemoryLayout.PathElement.groupElement("uv0_offset");
        UV0_FORMAT_PATH = MemoryLayout.PathElement.groupElement("uv0_format");
        UV1_OFFSET_PATH = MemoryLayout.PathElement.groupElement("uv1_offset");
        UV1_FORMAT_PATH = MemoryLayout.PathElement.groupElement("uv1_format");
        UV2_OFFSET_PATH = MemoryLayout.PathElement.groupElement("uv2_offset");
        UV2_FORMAT_PATH = MemoryLayout.PathElement.groupElement("uv2_format");
        HAS_NORMAL_PATH = MemoryLayout.PathElement.groupElement("has_normal");
        HAS_COLOR_PATH = MemoryLayout.PathElement.groupElement("has_color");
        HAS_UV0_PATH = MemoryLayout.PathElement.groupElement("has_uv0");
        HAS_UV1_PATH = MemoryLayout.PathElement.groupElement("has_uv1");
        HAS_UV2_PATH = MemoryLayout.PathElement.groupElement("has_uv2");

        STRIDE_HANDLE = LAYOUT.varHandle(STRIDE_PATH);
        POSITION_OFFSET_HANDLE = LAYOUT.varHandle(POSITION_OFFSET_PATH);
        POSITION_FORMAT_HANDLE = LAYOUT.varHandle(POSITION_FORMAT_PATH);
        NORMAL_OFFSET_HANDLE = LAYOUT.varHandle(NORMAL_OFFSET_PATH);
        NORMAL_FORMAT_HANDLE = LAYOUT.varHandle(NORMAL_FORMAT_PATH);
        COLOR_OFFSET_HANDLE = LAYOUT.varHandle(COLOR_OFFSET_PATH);
        COLOR_FORMAT_HANDLE = LAYOUT.varHandle(COLOR_FORMAT_PATH);
        UV0_OFFSET_HANDLE = LAYOUT.varHandle(UV0_OFFSET_PATH);
        UV0_FORMAT_HANDLE = LAYOUT.varHandle(UV0_FORMAT_PATH);
        UV1_OFFSET_HANDLE = LAYOUT.varHandle(UV1_OFFSET_PATH);
        UV1_FORMAT_HANDLE = LAYOUT.varHandle(UV1_FORMAT_PATH);
        UV2_OFFSET_HANDLE = LAYOUT.varHandle(UV2_OFFSET_PATH);
        UV2_FORMAT_HANDLE = LAYOUT.varHandle(UV2_FORMAT_PATH);
        HAS_NORMAL_HANDLE = LAYOUT.varHandle(HAS_NORMAL_PATH);
        HAS_COLOR_HANDLE = LAYOUT.varHandle(HAS_COLOR_PATH);
        HAS_UV0_HANDLE = LAYOUT.varHandle(HAS_UV0_PATH);
        HAS_UV1_HANDLE = LAYOUT.varHandle(HAS_UV1_PATH);
        HAS_UV2_HANDLE = LAYOUT.varHandle(HAS_UV2_PATH);
    }

    private final ResourceScope resourceScope;
    private final MemorySegment memory;

    public VertexFormatNative() {
        this.resourceScope = ResourceScope.newSharedScope();
        this.memory = MemorySegment.allocateNative(LAYOUT, this.resourceScope);
        this.reset();
    }

    public void reset() {
        STRIDE_HANDLE.set(this.memory, 0);
        HAS_NORMAL_HANDLE.set(this.memory, false);
        HAS_COLOR_HANDLE.set(this.memory, false);
        HAS_UV0_HANDLE.set(this.memory, false);
        HAS_UV1_HANDLE.set(this.memory, false);
        HAS_UV2_HANDLE.set(this.memory, false);
    }

    public void setStride(int stride) {
        STRIDE_HANDLE.set(this.memory, stride);
    }

    public void setPosition(int offset, int format) {
        POSITION_OFFSET_HANDLE.set(this.memory, offset);
        POSITION_FORMAT_HANDLE.set(this.memory, format);
    }

    public void setNormal(int offset, int format) {
        NORMAL_OFFSET_HANDLE.set(this.memory, offset);
        NORMAL_FORMAT_HANDLE.set(this.memory, format);
        HAS_NORMAL_HANDLE.set(this.memory, true);
    }

    public void clearNormal() {
        HAS_NORMAL_HANDLE.set(this.memory, false);
    }

    public void setColor(int offset, int format) {
        COLOR_OFFSET_HANDLE.set(this.memory, offset);
        COLOR_FORMAT_HANDLE.set(this.memory, format);
        HAS_COLOR_HANDLE.set(this.memory, true);
    }

    public void clearColor() {
        HAS_COLOR_HANDLE.set(this.memory, false);
    }

    public void setUV0(int offset, int format) {
        UV0_OFFSET_HANDLE.set(this.memory, offset);
        UV0_FORMAT_HANDLE.set(this.memory, format);
        HAS_UV0_HANDLE.set(this.memory, true);
    }

    public void clearUV0() {
        HAS_UV0_HANDLE.set(this.memory, false);
    }

    public void setUV1(int offset, int format) {
        UV1_OFFSET_HANDLE.set(this.memory, offset);
        UV1_FORMAT_HANDLE.set(this.memory, format);
        HAS_UV1_HANDLE.set(this.memory, true);
    }

    public void clearUV1() {
        HAS_UV1_HANDLE.set(this.memory, false);
    }

    public void setUV2(int offset, int format) {
        UV2_OFFSET_HANDLE.set(this.memory, offset);
        UV2_FORMAT_HANDLE.set(this.memory, format);
        HAS_UV2_HANDLE.set(this.memory, true);
    }

    public void clearUV2() {
        HAS_UV2_HANDLE.set(this.memory, false);
    }

    public MemoryAddress getAddress() {
        return this.memory.address();
    }

    @Override
    public void close() throws Exception {
        this.resourceScope.close();
    }
}
