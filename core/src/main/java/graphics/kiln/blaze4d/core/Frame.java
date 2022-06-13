package graphics.kiln.blaze4d.core;

import graphics.kiln.blaze4d.core.natives.Natives;
import graphics.kiln.blaze4d.core.types.B4DUniformData;
import jdk.incubator.foreign.MemoryAddress;

public class Frame implements AutoCloseable {

    private final MemoryAddress handle;

    Frame(MemoryAddress handle) {
        this.handle = handle;
    }

    public void updateUniform(long shaderId, B4DUniformData data) {
        Natives.b4dPassUpdateUniform(this.handle, data.getAddress(), shaderId);
    }

    @Override
    public void close() throws Exception {
        Natives.b4dEndFrame(this.handle);
    }
}
