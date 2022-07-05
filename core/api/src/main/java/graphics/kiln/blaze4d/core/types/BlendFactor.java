package graphics.kiln.blaze4d.core.types;

public enum BlendFactor {
    ZERO(0),
    ONE(1),
    SRC_COLOR(2),
    ONE_MINUS_SRC_COLOR(3),
    DST_COLOR(4),
    ONE_MINUS_DST_COLOR(5),
    SRC_ALPHA(6),
    ONE_MINUS_SRC_ALPHA(7),
    DST_ALPHA(8),
    ONE_MINUS_DST_ALPHA(9);

    private final int value;

    BlendFactor(int value) {
        this.value = value;
    }

    public int getValue() {
        return this.value;
    }

    public static BlendFactor fromValue(int value) {
        return switch (value) {
            case 0 -> ZERO;
            case 1 -> ONE;
            case 2 -> SRC_COLOR;
            case 3 -> ONE_MINUS_SRC_COLOR;
            case 4 -> DST_COLOR;
            case 5 -> ONE_MINUS_DST_COLOR;
            case 6 -> SRC_ALPHA;
            case 7 -> ONE_MINUS_SRC_ALPHA;
            case 8 -> DST_ALPHA;
            case 9 -> ONE_MINUS_DST_ALPHA;
            default -> throw new IllegalArgumentException("Invalid blend factor value: " + value);
        };
    }

    public static BlendFactor fromGlBlendFunc(int factor) {
        return switch (factor) {
            case 0 -> ZERO;
            case 1 -> ONE;
            case 0x0300 -> SRC_COLOR;
            case 0x0301 -> ONE_MINUS_SRC_COLOR;
            case 0x0302 -> SRC_ALPHA;
            case 0x0303 -> ONE_MINUS_SRC_ALPHA;
            case 0x0304 -> DST_ALPHA;
            case 0x0305 -> ONE_MINUS_DST_ALPHA;
            case 0x0306 -> DST_COLOR;
            case 0x0307 -> ONE_MINUS_DST_COLOR;
            default -> throw new IllegalArgumentException("Invalid blend func: " + factor);
        };
    }
}
