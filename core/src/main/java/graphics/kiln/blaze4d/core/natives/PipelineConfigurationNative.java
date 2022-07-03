package graphics.kiln.blaze4d.core.natives;

import jdk.incubator.foreign.MemoryLayout;
import jdk.incubator.foreign.ValueLayout;

import java.lang.invoke.VarHandle;

public class PipelineConfigurationNative {
    public static final MemoryLayout LAYOUT;

    public static final MemoryLayout.PathElement DEPTH_TEST_ENABLE_PATH;
    public static final MemoryLayout.PathElement DEPTH_COMPARE_OP_PATH;
    public static final MemoryLayout.PathElement DEPTH_WRITE_ENABLE_PATH;
    public static final MemoryLayout.PathElement BLEND_ENABLE_PATH;
    public static final MemoryLayout.PathElement BLEND_COLOR_OP_PATH;
    public static final MemoryLayout.PathElement BLEND_COLOR_SRC_FACTOR_PATH;
    public static final MemoryLayout.PathElement BLEND_COLOR_DST_FACTOR_PATH;
    public static final MemoryLayout.PathElement BLEND_ALPHA_OP_PATH;
    public static final MemoryLayout.PathElement BLEND_ALPHA_SRC_FACTOR_PATH;
    public static final MemoryLayout.PathElement BLEND_ALPHA_DST_FACTOR_PATH;

    public static final VarHandle DEPTH_TEST_ENABLE_HANDLE;
    public static final VarHandle DEPTH_COMPARE_OP_HANDLE;
    public static final VarHandle DEPTH_WRITE_ENABLE_HANDLE;
    public static final VarHandle BLEND_ENABLE_HANDLE;
    public static final VarHandle BLEND_COLOR_OP_HANDLE;
    public static final VarHandle BLEND_COLOR_SRC_FACTOR_HANDLE;
    public static final VarHandle BLEND_COLOR_DST_FACTOR_HANDLE;
    public static final VarHandle BLEND_ALPHA_OP_HANDLE;
    public static final VarHandle BLEND_ALPHA_SRC_FACTOR_HANDLE;
    public static final VarHandle BLEND_ALPHA_DST_FACTOR_HANDLE;

    static {
        LAYOUT = MemoryLayout.structLayout(
                ValueLayout.JAVA_INT.withName("depth_test_enable"),
                ValueLayout.JAVA_INT.withName("depth_compare_op"),
                ValueLayout.JAVA_INT.withName("depth_write_enable"),
                ValueLayout.JAVA_INT.withName("blend_enable"),
                ValueLayout.JAVA_INT.withName("blend_color_op"),
                ValueLayout.JAVA_INT.withName("blend_color_src_factor"),
                ValueLayout.JAVA_INT.withName("blend_color_dst_factor"),
                ValueLayout.JAVA_INT.withName("blend_alpha_op"),
                ValueLayout.JAVA_INT.withName("blend_alpha_src_factor"),
                ValueLayout.JAVA_INT.withName("blend_alpha_dst_factor")
        );

        DEPTH_TEST_ENABLE_PATH = MemoryLayout.PathElement.groupElement("depth_test_enable");
        DEPTH_COMPARE_OP_PATH = MemoryLayout.PathElement.groupElement("depth_compare_op");
        DEPTH_WRITE_ENABLE_PATH = MemoryLayout.PathElement.groupElement("depth_write_enable");
        BLEND_ENABLE_PATH = MemoryLayout.PathElement.groupElement("blend_enable");
        BLEND_COLOR_OP_PATH = MemoryLayout.PathElement.groupElement("blend_color_op");
        BLEND_COLOR_SRC_FACTOR_PATH = MemoryLayout.PathElement.groupElement("blend_color_src_factor");
        BLEND_COLOR_DST_FACTOR_PATH = MemoryLayout.PathElement.groupElement("blend_color_dst_factor");
        BLEND_ALPHA_OP_PATH = MemoryLayout.PathElement.groupElement("blend_alpha_op");
        BLEND_ALPHA_SRC_FACTOR_PATH = MemoryLayout.PathElement.groupElement("blend_alpha_src_factor");
        BLEND_ALPHA_DST_FACTOR_PATH = MemoryLayout.PathElement.groupElement("blend_alpha_dst_factor");

        DEPTH_TEST_ENABLE_HANDLE = LAYOUT.varHandle(DEPTH_TEST_ENABLE_PATH);
        DEPTH_COMPARE_OP_HANDLE = LAYOUT.varHandle(DEPTH_COMPARE_OP_PATH);
        DEPTH_WRITE_ENABLE_HANDLE = LAYOUT.varHandle(DEPTH_WRITE_ENABLE_PATH);
        BLEND_ENABLE_HANDLE = LAYOUT.varHandle(BLEND_ENABLE_PATH);
        BLEND_COLOR_OP_HANDLE = LAYOUT.varHandle(BLEND_COLOR_OP_PATH);
        BLEND_COLOR_SRC_FACTOR_HANDLE = LAYOUT.varHandle(BLEND_COLOR_SRC_FACTOR_PATH);
        BLEND_COLOR_DST_FACTOR_HANDLE = LAYOUT.varHandle(BLEND_COLOR_DST_FACTOR_PATH);
        BLEND_ALPHA_OP_HANDLE = LAYOUT.varHandle(BLEND_ALPHA_OP_PATH);
        BLEND_ALPHA_SRC_FACTOR_HANDLE = LAYOUT.varHandle(BLEND_ALPHA_SRC_FACTOR_PATH);
        BLEND_ALPHA_DST_FACTOR_HANDLE = LAYOUT.varHandle(BLEND_ALPHA_DST_FACTOR_PATH);
    }
}