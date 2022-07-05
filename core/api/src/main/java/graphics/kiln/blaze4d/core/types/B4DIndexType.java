package graphics.kiln.blaze4d.core.types;

public enum B4DIndexType {
    UINT16(0),
    UINT32(1);

    private final int value;

    B4DIndexType(int value) {
        this.value = value;
    }

    public int getValue() {
        return this.value;
    }

    public static B4DIndexType fromValue(int value) {
        switch (value) {
            case 0 -> {
                return B4DIndexType.UINT16;
            }
            case 1 -> {
                return B4DIndexType.UINT32;
            }
            default ->
                throw new RuntimeException("Invalid index type value " + value);
        }
    }
}
