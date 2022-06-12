package graphics.kiln.blaze4d.core.structs;

import jdk.incubator.foreign.MemoryLayout;
import jdk.incubator.foreign.ValueLayout;

public class Vec2u32 {
    public static final MemoryLayout LAYOUT;

    static {
        LAYOUT = MemoryLayout.structLayout(
                ValueLayout.JAVA_INT,
                ValueLayout.JAVA_INT
        );
    }


}
