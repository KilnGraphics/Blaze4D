package graphics.kiln.blaze4d.core;

public enum McUniform {
    MODEL_VIEW_MATRIX(1L),
    PROJECTION_MATRIX(1L << 1),
    INVERSE_VIEW_ROTATION_MATRIX(1L << 2),
    TEXTURE_MATRIX(1L << 3),
    SCREEN_SIZE(1L << 4),
    COLOR_MODULATOR(1L << 5),
    LIGHT0_DIRECTION(1L << 6),
    LIGHT1_DIRECTION(1L << 7),
    FOG_START(1L << 8),
    FOG_END(1L << 9),
    FOG_COLOR(1L << 10),
    FOG_SHAPE(1L << 11),
    LINE_WIDTH(1L << 12),
    GAME_TIME(1L << 13),
    CHUNK_OFFSET(1L << 14);

    private final long value;

    McUniform(long value) {
        this.value = value;
    }

    public long getValue() {
        return this.value;
    }
}
