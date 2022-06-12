package graphics.kiln.blaze4d.core;

import graphics.kiln.blaze4d.core.natives.Natives;
import jdk.incubator.foreign.MemoryAddress;

public class Frame implements AutoCloseable {

    private final MemoryAddress handle;

    Frame(MemoryAddress handle) {
        this.handle = handle;
    }

    @Override
    public void close() throws Exception {
        Natives.b4dEndFrame(this.handle);
    }
}
