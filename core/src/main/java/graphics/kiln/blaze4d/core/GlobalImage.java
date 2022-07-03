package graphics.kiln.blaze4d.core;

import graphics.kiln.blaze4d.core.natives.Natives;
import graphics.kiln.blaze4d.core.types.B4DImageData;
import jdk.incubator.foreign.MemoryAddress;

public class GlobalImage implements AutoCloseable {

    private final MemoryAddress handle;

    GlobalImage(MemoryAddress handle) {
        this.handle = handle;
    }

    public void update(B4DImageData data) {
        Natives.b4DUpdateGlobalImage(this.handle, data.getAddress(), 1);
    }

    MemoryAddress getHandle() {
        return this.handle;
    }

    @Override
    public void close() throws Exception {
        Natives.b4dDestroyGlobalImage(this.handle);
    }
}
