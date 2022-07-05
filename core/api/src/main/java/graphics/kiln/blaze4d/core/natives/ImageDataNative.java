package graphics.kiln.blaze4d.core.natives;

import jdk.incubator.foreign.MemoryLayout;
import jdk.incubator.foreign.ValueLayout;

import java.lang.invoke.VarHandle;

public class ImageDataNative {
    public static final MemoryLayout LAYOUT;

    public static final MemoryLayout.PathElement DATA_PTR_PATH;
    public static final MemoryLayout.PathElement DATA_LEN_PATH;
    public static final MemoryLayout.PathElement ROW_STRIDE_PATH;
    public static final MemoryLayout.PathElement OFFSET_PATH;
    public static final MemoryLayout.PathElement EXTENT_PATH;

    public static final VarHandle DATA_PTR_HANDLE;
    public static final VarHandle DATA_LEN_HANDLE;
    public static final VarHandle ROW_STRIDE_HANDLE;
    public static final VarHandle OFFSET_HANDLE;
    public static final VarHandle EXTENT_HANDLE;

    static {
        LAYOUT = MemoryLayout.structLayout(
                ValueLayout.ADDRESS.withName("data_ptr"),
                Natives.getSizeLayout().withName("data_len"),
                ValueLayout.JAVA_INT.withName("row_stride"),
                MemoryLayout.sequenceLayout(2, ValueLayout.JAVA_INT).withName("offset"),
                MemoryLayout.sequenceLayout(2, ValueLayout.JAVA_INT).withName("extent")
        );

        DATA_PTR_PATH = MemoryLayout.PathElement.groupElement("data_ptr");
        DATA_LEN_PATH = MemoryLayout.PathElement.groupElement("data_len");
        ROW_STRIDE_PATH = MemoryLayout.PathElement.groupElement("row_stride");
        OFFSET_PATH = MemoryLayout.PathElement.groupElement("offset");
        EXTENT_PATH = MemoryLayout.PathElement.groupElement("extent");

        DATA_PTR_HANDLE = LAYOUT.varHandle(DATA_PTR_PATH);
        DATA_LEN_HANDLE = LAYOUT.varHandle(DATA_LEN_PATH);
        ROW_STRIDE_HANDLE = LAYOUT.varHandle(ROW_STRIDE_PATH);
        OFFSET_HANDLE = LAYOUT.varHandle(OFFSET_PATH, MemoryLayout.PathElement.sequenceElement());
        EXTENT_HANDLE = LAYOUT.varHandle(EXTENT_PATH, MemoryLayout.PathElement.sequenceElement());
    }
}
