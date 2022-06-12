package graphics.kiln.blaze4d.core.natives;

import jdk.incubator.foreign.*;

import java.lang.invoke.VarHandle;

public class MeshDataNative implements AutoCloseable {
    public static final MemoryLayout LAYOUT;

    public static final MemoryLayout.PathElement VERTEX_DATA_PTR_PATH;
    public static final MemoryLayout.PathElement VERTEX_DATA_LEN_PATH;
    public static final MemoryLayout.PathElement INDEX_DATA_PTR_PATH;
    public static final MemoryLayout.PathElement INDEX_DATA_LEN_PATH;
    public static final MemoryLayout.PathElement VERTEX_STRIDE_PATH;
    public static final MemoryLayout.PathElement INDEX_COUNT_PATH;
    public static final MemoryLayout.PathElement INDEX_TYPE_PATH;
    public static final MemoryLayout.PathElement PRIMITIVE_TOPOLOGY_PATH;

    public static final VarHandle VERTEX_DATA_PTR_HANDLE;
    public static final VarHandle VERTEX_DATA_LEN_HANDLE;
    public static final VarHandle INDEX_DATA_PTR_HANDLE;
    public static final VarHandle INDEX_DATA_LEN_HANDLE;
    public static final VarHandle VERTEX_STRIDE_HANDLE;
    public static final VarHandle INDEX_COUNT_HANDLE;
    public static final VarHandle INDEX_TYPE_HANDLE;
    public static final VarHandle PRIMITIVE_TOPOLOGY_HANDLE;

    static {
        LAYOUT = MemoryLayout.structLayout(
                ValueLayout.ADDRESS.withName("vertex_data_ptr"),
                Natives.getSizeLayout().withName("vertex_data_len"),
                ValueLayout.ADDRESS.withName("index_data_ptr"),
                Natives.getSizeLayout().withName("index_data_len"),
                ValueLayout.JAVA_INT.withName("vertex_stride"),
                ValueLayout.JAVA_INT.withName("index_count"),
                ValueLayout.JAVA_INT.withName("index_type"),
                ValueLayout.JAVA_INT.withName("primitive_topology")
        );

        VERTEX_DATA_PTR_PATH = MemoryLayout.PathElement.groupElement("vertex_data_ptr");
        VERTEX_DATA_LEN_PATH = MemoryLayout.PathElement.groupElement("vertex_data_len");
        INDEX_DATA_PTR_PATH = MemoryLayout.PathElement.groupElement("index_data_ptr");
        INDEX_DATA_LEN_PATH = MemoryLayout.PathElement.groupElement("index_data_len");
        VERTEX_STRIDE_PATH = MemoryLayout.PathElement.groupElement("vertex_stride");
        INDEX_COUNT_PATH = MemoryLayout.PathElement.groupElement("index_count");
        INDEX_TYPE_PATH = MemoryLayout.PathElement.groupElement("index_type");
        PRIMITIVE_TOPOLOGY_PATH = MemoryLayout.PathElement.groupElement("primitive_topology");

        VERTEX_DATA_PTR_HANDLE = LAYOUT.varHandle(VERTEX_DATA_PTR_PATH);
        VERTEX_DATA_LEN_HANDLE = LAYOUT.varHandle(VERTEX_DATA_LEN_PATH);
        INDEX_DATA_PTR_HANDLE = LAYOUT.varHandle(INDEX_DATA_PTR_PATH);
        INDEX_DATA_LEN_HANDLE = LAYOUT.varHandle(INDEX_DATA_LEN_PATH);
        VERTEX_STRIDE_HANDLE = LAYOUT.varHandle(VERTEX_STRIDE_PATH);
        INDEX_COUNT_HANDLE = LAYOUT.varHandle(INDEX_COUNT_PATH);
        INDEX_TYPE_HANDLE = LAYOUT.varHandle(INDEX_TYPE_PATH);
        PRIMITIVE_TOPOLOGY_HANDLE = LAYOUT.varHandle(PRIMITIVE_TOPOLOGY_PATH);
    }

    private final ResourceScope resourceScope;
    private final MemorySegment memory;

    public MeshDataNative() {
        this.resourceScope = ResourceScope.newSharedScope();
        this.memory = MemorySegment.allocateNative(LAYOUT, this.resourceScope);
    }

    public void setVertexData(MemoryAddress data, long dataLen) {
        VERTEX_DATA_PTR_HANDLE.set(this.memory, data);
        VERTEX_DATA_LEN_HANDLE.set(this.memory, dataLen);
    }

    public void setIndexData(MemoryAddress data, long dataLen) {
        INDEX_DATA_PTR_HANDLE.set(this.memory, data);
        INDEX_DATA_LEN_HANDLE.set(this.memory, dataLen);
    }

    public void setVertexStride(int vertexStride) {
        VERTEX_STRIDE_HANDLE.set(this.memory, vertexStride);
    }

    public void setIndexCount(int indexCount) {
        INDEX_COUNT_HANDLE.set(this.memory, indexCount);
    }

    public void setIndexType(int indexType) {
        INDEX_TYPE_HANDLE.set(this.memory, indexType);
    }

    public void setPrimitiveTopology(int primitiveTopology) {
        PRIMITIVE_TOPOLOGY_HANDLE.set(this.memory, primitiveTopology);
    }

    public MemoryAddress getAddress() {
        return this.memory.address();
    }

    @Override
    public void close() throws Exception {
        this.resourceScope.close();
    }
}
