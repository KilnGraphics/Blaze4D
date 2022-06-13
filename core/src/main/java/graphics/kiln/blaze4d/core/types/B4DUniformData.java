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

    public void setModelViewMatrix(float m00, float m01, float m02, float m03, float m10, float m11, float m12, float m13, float m20, float m21, float m22, float m23, float m30, float m31, float m32, float m33) {
        McUniformDataNative.UNIFORM_HANDLE.set(this.memory, B4DUniform.MODEL_VIEW_MATRIX.getValue());
        this.setMat4f32(m00, m01, m02, m03, m10, m11, m12, m13, m20, m21, m22, m23, m30, m31, m32, m33);
    }

    public void setProjectionMatrix(float m00, float m01, float m02, float m03, float m10, float m11, float m12, float m13, float m20, float m21, float m22, float m23, float m30, float m31, float m32, float m33) {
        McUniformDataNative.UNIFORM_HANDLE.set(this.memory, B4DUniform.PROJECTION_MATRIX.getValue());
        this.setMat4f32(m00, m01, m02, m03, m10, m11, m12, m13, m20, m21, m22, m23, m30, m31, m32, m33);
    }

    public void setChunkOffset(float x, float y, float z) {
        McUniformDataNative.UNIFORM_HANDLE.set(this.memory, B4DUniform.CHUNK_OFFSET.getValue());
        this.setVec3f32(x, y, z);
    }

    public MemoryAddress getAddress() {
        return this.memory.address();
    }

    @Override
    public void close() throws Exception {
        this.resourceScope.close();
    }

    private void setVec3f32(float x, float y, float z) {
        McUniformDataNative.PAYLOAD_VEC3F32_HANDLE.set(this.memory, 0, x);
        McUniformDataNative.PAYLOAD_VEC3F32_HANDLE.set(this.memory, 1, y);
        McUniformDataNative.PAYLOAD_VEC3F32_HANDLE.set(this.memory, 2, z);
    }

    private void setMat4f32(float m00, float m01, float m02, float m03, float m10, float m11, float m12, float m13, float m20, float m21, float m22, float m23, float m30, float m31, float m32, float m33) {
        McUniformDataNative.PAYLOAD_MAT4F32_HANDLE.set(this.memory, 0, m00);
        McUniformDataNative.PAYLOAD_MAT4F32_HANDLE.set(this.memory, 1, m10);
        McUniformDataNative.PAYLOAD_MAT4F32_HANDLE.set(this.memory, 2, m20);
        McUniformDataNative.PAYLOAD_MAT4F32_HANDLE.set(this.memory, 3, m30);
        McUniformDataNative.PAYLOAD_MAT4F32_HANDLE.set(this.memory, 4, m01);
        McUniformDataNative.PAYLOAD_MAT4F32_HANDLE.set(this.memory, 5, m11);
        McUniformDataNative.PAYLOAD_MAT4F32_HANDLE.set(this.memory, 6, m21);
        McUniformDataNative.PAYLOAD_MAT4F32_HANDLE.set(this.memory, 7, m31);
        McUniformDataNative.PAYLOAD_MAT4F32_HANDLE.set(this.memory, 8, m02);
        McUniformDataNative.PAYLOAD_MAT4F32_HANDLE.set(this.memory, 9, m12);
        McUniformDataNative.PAYLOAD_MAT4F32_HANDLE.set(this.memory, 10, m22);
        McUniformDataNative.PAYLOAD_MAT4F32_HANDLE.set(this.memory, 11, m32);
        McUniformDataNative.PAYLOAD_MAT4F32_HANDLE.set(this.memory, 12, m03);
        McUniformDataNative.PAYLOAD_MAT4F32_HANDLE.set(this.memory, 13, m13);
        McUniformDataNative.PAYLOAD_MAT4F32_HANDLE.set(this.memory, 14, m23);
        McUniformDataNative.PAYLOAD_MAT4F32_HANDLE.set(this.memory, 15, m33);
    }
}
