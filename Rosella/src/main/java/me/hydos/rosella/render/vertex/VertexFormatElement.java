package me.hydos.rosella.render.vertex;

import java.util.Objects;

public final class VertexFormatElement {
    private final int vkType;
    private final int byteLength;

    VertexFormatElement(int vkType, int byteLength) {
        this.vkType = vkType;
        this.byteLength = byteLength;
    }

    public int getVkType() {
        return vkType;
    }

    public int getByteLength() {
        return byteLength;
    }

    @Override
    public boolean equals(Object obj) {
        if (obj == this) return true;
        if (obj == null || obj.getClass() != this.getClass()) return false;
        var that = (VertexFormatElement) obj;
        return this.vkType == that.vkType &&
                this.byteLength == that.byteLength;
    }

    @Override
    public int hashCode() {
        return Objects.hash(vkType, byteLength);
    }

    @Override
    public String toString() {
        return "VertexFormatElement[" +
                "vkId=" + vkType + ", " +
                "size=" + byteLength + ']';
    }


    enum DataType {
        FLOAT(Float.BYTES),
        UBYTE(Byte.BYTES),
        BYTE(Byte.BYTES),
        USHORT(Short.BYTES),
        SHORT(Short.BYTES),
        UINT(Integer.BYTES),
        INT(Integer.BYTES);

        private final int byteLength;

        DataType(int byteLength) {
            this.byteLength = byteLength;
        }

        public int getByteLength() {
            return byteLength;
        }
    }
}
