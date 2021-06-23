package me.hydos.aftermath.callback;

import org.jetbrains.annotations.Nullable;
import org.lwjgl.system.Callback;

import static org.lwjgl.system.MemoryUtil.NULL;


public abstract class GPUCrashDumpDescriptionCallback extends Callback implements GpuCrashDumpDescriptionCallbackI {
    public static GPUCrashDumpDescriptionCallback create(long functionPointer) {
        GpuCrashDumpDescriptionCallbackI instance = Callback.get(functionPointer);
        return instance instanceof GPUCrashDumpDescriptionCallback
                ? (GPUCrashDumpDescriptionCallback) instance
                : new Container(functionPointer, instance);
    }

    @Nullable
    public static GPUCrashDumpDescriptionCallback createSafe(long functionPointer) {
        return functionPointer == NULL ? null : create(functionPointer);
    }

    public static GPUCrashDumpDescriptionCallback create(GpuCrashDumpDescriptionCallbackI instance) {
        return instance instanceof GPUCrashDumpDescriptionCallback
                ? (GPUCrashDumpDescriptionCallback)instance
                : new Container(instance.address(), instance);
    }

    protected GPUCrashDumpDescriptionCallback() {
        super(CIF);
    }

    GPUCrashDumpDescriptionCallback(long functionPointer) {
        super(functionPointer);
    }

    private static final class Container extends GPUCrashDumpDescriptionCallback {
        private final GpuCrashDumpDescriptionCallbackI delegate;

        Container(long functionPointer, GpuCrashDumpDescriptionCallbackI delegate) {
            super(functionPointer);
            this.delegate = delegate;
        }


        @Override
        public void invoke(long addValue, long pUserData) {
            delegate.invoke(addValue, pUserData);
        }
    }
}
