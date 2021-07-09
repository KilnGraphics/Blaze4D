package me.hydos.rosella.render.vertex;

public record VertexFormatElement(int vkType, int byteLength) {

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
