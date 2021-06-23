package me.hydos.aftermath.callback;

import org.jetbrains.annotations.Nullable;
import org.lwjgl.system.Callback;

import java.io.IOException;

import static org.lwjgl.system.MemoryUtil.NULL;


public abstract class GPUCrashDumpCallback extends Callback implements GPUCrashDumpCallbackI {
    public static GPUCrashDumpCallback create(long functionPointer) {
        GPUCrashDumpCallbackI instance = Callback.get(functionPointer);
        return instance instanceof GPUCrashDumpCallback
                ? (GPUCrashDumpCallback) instance
                : new Container(functionPointer, instance);
    }

    @Nullable
    public static GPUCrashDumpCallback createSafe(long functionPointer) {
        return functionPointer == NULL ? null : create(functionPointer);
    }

    public static GPUCrashDumpCallback create(GPUCrashDumpCallbackI instance) {
        return instance instanceof GPUCrashDumpCallback
                ? (GPUCrashDumpCallback)instance
                : new Container(instance.address(), instance);
    }

    protected GPUCrashDumpCallback() {
        super(CIF);
    }

    GPUCrashDumpCallback(long functionPointer) {
        super(functionPointer);
    }

    private static final class Container extends GPUCrashDumpCallback {
        private final GPUCrashDumpCallbackI delegate;

        Container(long functionPointer, GPUCrashDumpCallbackI delegate) {
            super(functionPointer);
            this.delegate = delegate;
        }


        @Override
        public void invoke(long pGpuCrashDump, int gpuCrashDumpSize, long pUserData) throws IOException {
            delegate.invoke(pGpuCrashDump, gpuCrashDumpSize, pUserData);
        }
    }
}
