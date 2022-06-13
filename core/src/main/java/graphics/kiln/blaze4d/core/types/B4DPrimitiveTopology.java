package graphics.kiln.blaze4d.core.types;

public enum B4DPrimitiveTopology {
    POINT_LIST(0),
    LINE_LIST(1),
    LINE_STRIP(2),
    TRIANGLE_LIST(3),
    TRIANGLE_STRIP(4),
    TRIANGLE_FAN(5);

    private final int value;

    B4DPrimitiveTopology(int value) {
        this.value = value;
    }

    public int getValue() {
        return this.value;
    }

    public static B4DPrimitiveTopology fromRaw(int value) {
        switch (value) {
            case 0 -> {
                return B4DPrimitiveTopology.POINT_LIST;
            }
            case 1 -> {
                return B4DPrimitiveTopology.LINE_LIST;
            }
            case 2 -> {
                return B4DPrimitiveTopology.LINE_STRIP;
            }
            case 3 -> {
                return B4DPrimitiveTopology.TRIANGLE_LIST;
            }
            case 4 -> {
                return B4DPrimitiveTopology.TRIANGLE_STRIP;
            }
            case 5 -> {
                return B4DPrimitiveTopology.TRIANGLE_FAN;
            }
            default ->
                throw new RuntimeException("Invalid primitive topology value " + value);
        }
    }
}
