package graphics.kiln.blaze4d.core.types;

import graphics.kiln.blaze4d.core.natives.ImageDataNative;
import jdk.incubator.foreign.MemoryAddress;
import jdk.incubator.foreign.MemorySegment;
import jdk.incubator.foreign.ResourceScope;

public class B4DImageData implements AutoCloseable {

    private final ResourceScope resourceScope;
    private final MemorySegment memory;

    public B4DImageData() {
        this.resourceScope = ResourceScope.newSharedScope();
        this.memory = MemorySegment.allocateNative(ImageDataNative.LAYOUT, this.resourceScope);
    }

    private void setData(MemoryAddress data, long dataLen) {
        ImageDataNative.DATA_PTR_HANDLE.set(this.memory, data);
        ImageDataNative.DATA_LEN_HANDLE.set(this.memory, dataLen);
    }

    public void setData(long dataPtr, long dataLen) {
        this.setData(MemoryAddress.ofLong(dataPtr), dataLen);
    }

    public void setRowStride(int rowStride) {
        ImageDataNative.ROW_STRIDE_HANDLE.set(this.memory, rowStride);
    }

    public void setOffset(int x, int y) {
        ImageDataNative.OFFSET_HANDLE.set(this.memory, 0, x);
        ImageDataNative.OFFSET_HANDLE.set(this.memory, 1, y);
    }

    public void setExtent(int x, int y) {
        ImageDataNative.EXTENT_HANDLE.set(this.memory, 0, x);
        ImageDataNative.EXTENT_HANDLE.set(this.memory, 1, y);
    }

    public MemoryAddress getAddress() {
        return this.memory.address();
    }

    @Override
    public void close() throws Exception {
        this.resourceScope.close();
    }
}
