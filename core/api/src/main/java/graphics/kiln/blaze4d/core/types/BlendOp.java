package graphics.kiln.blaze4d.core.types;

public enum BlendOp {
    ADD(0),
    SUBTRACT(1),
    REVERSE_SUBTRACT(2),
    MIN(3),
    MAX(4);

    private final int value;

    BlendOp(int value) {
        this.value = value;
    }

    public int getValue() {
        return this.value;
    }

    public static BlendOp fromValue(int value) {
        return switch (value) {
            case 0 -> ADD;
            case 1 -> SUBTRACT;
            case 2 -> REVERSE_SUBTRACT;
            case 3 -> MIN;
            case 4 -> MAX;
            default -> throw new IllegalArgumentException("Invalid blend op value: " + value);
        };
    }

    public static BlendOp fromGlBlendEquation(int glEquation) {
        return switch (glEquation) {
            case 0x8006 -> ADD;
            case 0x8007 -> MIN;
            case 0x8008 -> MAX;
            case 0x800A -> SUBTRACT;
            case 0x800B -> REVERSE_SUBTRACT;
            default -> throw new IllegalArgumentException("Invalid blend equation: " + glEquation);
        };
    }
}
