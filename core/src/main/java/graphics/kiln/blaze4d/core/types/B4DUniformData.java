package graphics.kiln.blaze4d.core.types;

import graphics.kiln.blaze4d.core.natives.McUniformDataNative;
import jdk.incubator.foreign.MemoryAddress;
import jdk.incubator.foreign.MemorySegment;
import jdk.incubator.foreign.ResourceScope;

public class B4DUniformData implements AutoCloseable {

    private final ResourceScope resourceScope;
    private final MemorySegment memory;

    public B4DUniformData() {
        this.resourceScope = ResourceScope.newSharedScope();
        this.memory = MemorySegment.allocateNative(McUniformDataNative.LAYOUT, this.resourceScope);
    }

    public void setChunkOffset(float x, float y, float z) {
        McUniformDataNative.UNIFORM_HANDLE.set(this.memory, B4DUniform.CHUNK_OFFSET.getValue());
        McUniformDataNative.PAYLOAD_VEC3F32_HANDLE.set(this.memory, 0, x);
        McUniformDataNative.PAYLOAD_VEC3F32_HANDLE.set(this.memory, 1, y);
        McUniformDataNative.PAYLOAD_VEC3F32_HANDLE.set(this.memory, 2, z);
    }

    public MemoryAddress getAddress() {
        return this.memory.address();
    }

    @Override
    public void close() throws Exception {
        this.resourceScope.close();
    }
}
