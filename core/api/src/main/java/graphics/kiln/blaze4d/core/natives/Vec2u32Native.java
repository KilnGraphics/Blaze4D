package graphics.kiln.blaze4d.core.natives;

import jdk.incubator.foreign.MemoryLayout;
import jdk.incubator.foreign.ValueLayout;

public class Vec2u32Native {
    public static final MemoryLayout LAYOUT;

    static {
        LAYOUT = MemoryLayout.sequenceLayout(2, ValueLayout.JAVA_INT);
    }
}
