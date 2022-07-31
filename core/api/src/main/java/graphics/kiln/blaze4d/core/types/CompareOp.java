package graphics.kiln.blaze4d.core.types;

public enum CompareOp {
    NEVER(0),
    LESS(1),
    EQUAL(2),
    LESS_OR_EQUAL(3),
    GREATER(4),
    NOT_EQUAL(5),
    GREATER_OR_EQUAL(6),
    ALWAYS(7);

    private final int value;

    CompareOp(int value) {
        this.value = value;
    }

    public int getValue() {
        return this.value;
    }

    public static CompareOp fromValue(int value) {
        return switch (value) {
            case 0 -> NEVER;
            case 1 -> LESS;
            case 2 -> EQUAL;
            case 3 -> LESS_OR_EQUAL;
            case 4 -> GREATER;
            case 5 -> NOT_EQUAL;
            case 6 -> GREATER_OR_EQUAL;
            case 7 -> ALWAYS;
            default -> throw new IllegalStateException("Invalid compare op value: " + value);
        };
    }

    public static CompareOp fromGlDepthFunc(int glFunc) {
        return switch (glFunc) {
            case 0x0200 -> NEVER;
            case 0x0201 -> LESS;
            case 0x0202 -> EQUAL;
            case 0x0203 -> LESS_OR_EQUAL;
            case 0x0204 -> GREATER;
            case 0x0205 -> NOT_EQUAL;
            case 0x0206 -> GREATER_OR_EQUAL;
            case 0x0207 -> ALWAYS;
            default -> throw new IllegalStateException("Invalid depth function value: " + glFunc);
        };
    }
}
