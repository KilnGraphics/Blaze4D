package graphics.kiln.blaze4d.core;

import graphics.kiln.blaze4d.core.natives.Natives;
import jdk.incubator.foreign.MemoryAddress;

public class GlobalImage implements AutoCloseable {

    private final MemoryAddress handle;

    GlobalImage(MemoryAddress handle) {
        this.handle = handle;
    }

    MemoryAddress getHandle() {
        return this.handle;
    }

    @Override
    public void close() throws Exception {
        Natives.b4dDestroyGlobalImage(this.handle);
    }
}
