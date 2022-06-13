package graphics.kiln.blaze4d.core.types;

import graphics.kiln.blaze4d.core.natives.MeshDataNative;
import jdk.incubator.foreign.MemoryAddress;
import jdk.incubator.foreign.MemorySegment;
import jdk.incubator.foreign.ResourceScope;

/**
 * Wrapper class around the native CMeshData type.
 */
public class B4DMeshData implements AutoCloseable {

    private final ResourceScope resourceScope;
    private final MemorySegment memory;

    /**
     * Allocates a new mesh data instance with native backing memory.
     * All data will be uninitialized.
     */
    public B4DMeshData() {
        this.resourceScope = ResourceScope.newSharedScope();
        this.memory = MemorySegment.allocateNative(MeshDataNative.LAYOUT, this.resourceScope);
    }

    public void setVertexData(MemoryAddress data, long dataLen) {
        MeshDataNative.VERTEX_DATA_PTR_HANDLE.set(this.memory, data);
        MeshDataNative.VERTEX_DATA_LEN_HANDLE.set(this.memory, dataLen);
    }

    public void setVertexData(long dataPtr, long dataLen) {
        this.setVertexData(MemoryAddress.ofLong(dataPtr), dataLen);
    }

    public MemoryAddress getVertexDataPtr() {
        return (MemoryAddress) MeshDataNative.VERTEX_DATA_PTR_HANDLE.get(this.memory);
    }

    public long getVertexDataLen() {
        return (long) MeshDataNative.VERTEX_DATA_LEN_HANDLE.get(this.memory);
    }

    public void setIndexData(MemoryAddress data, long dataLen) {
        MeshDataNative.INDEX_DATA_PTR_HANDLE.set(this.memory, data);
        MeshDataNative.INDEX_DATA_LEN_HANDLE.set(this.memory, dataLen);
    }

    public void setIndexData(long dataPtr, long dataLen) {
        this.setIndexData(MemoryAddress.ofLong(dataPtr), dataLen);
    }

    public MemoryAddress getIndexDataPtr() {
        return (MemoryAddress) MeshDataNative.INDEX_DATA_PTR_HANDLE.get(this.memory);
    }

    public long getIndexDataLen() {
        return (long) MeshDataNative.INDEX_DATA_LEN_HANDLE.get(this.memory);
    }

    public void setVertexStride(int vertexStride) {
        MeshDataNative.VERTEX_STRIDE_HANDLE.set(this.memory, vertexStride);
    }

    public int getVertexStride() {
        return (int) MeshDataNative.VERTEX_STRIDE_HANDLE.get(this.memory);
    }

    public void setIndexCount(int indexCount) {
        MeshDataNative.INDEX_COUNT_HANDLE.set(this.memory, indexCount);
    }

    public int getIndexCount() {
        return (int) MeshDataNative.INDEX_COUNT_HANDLE.get(this.memory);
    }

    public void setIndexType(B4DIndexType type) {
        MeshDataNative.INDEX_TYPE_HANDLE.set(this.memory, type.getValue());
    }

    public B4DIndexType getIndexType() {
        return B4DIndexType.fromValue((int) MeshDataNative.INDEX_TYPE_HANDLE.get(this.memory));
    }

    public void setIndexTypeRaw(int indexType) {
        MeshDataNative.INDEX_TYPE_HANDLE.set(this.memory, indexType);
    }

    public int getIndexTypeRaw() {
        return (int) MeshDataNative.INDEX_TYPE_HANDLE.get(this.memory);
    }

    public void setPrimitiveTopology(B4DPrimitiveTopology primitiveTopology) {
        MeshDataNative.PRIMITIVE_TOPOLOGY_HANDLE.set(this.memory, primitiveTopology.getValue());
    }

    public B4DPrimitiveTopology getPrimitiveTopology() {
        return B4DPrimitiveTopology.fromRaw((int) MeshDataNative.PRIMITIVE_TOPOLOGY_HANDLE.get(this.memory));
    }

    public void setPrimitiveTopologyRaw(int primitiveTopology) {
        MeshDataNative.PRIMITIVE_TOPOLOGY_HANDLE.set(this.memory, primitiveTopology);
    }

    public int getPrimitiveTopologyRaw() {
        return (int) MeshDataNative.PRIMITIVE_TOPOLOGY_HANDLE.get(this.memory);
    }

    public MemoryAddress getAddress() {
        return this.memory.address();
    }

    @Override
    public void close() throws Exception {
        this.resourceScope.close();
    }
}
