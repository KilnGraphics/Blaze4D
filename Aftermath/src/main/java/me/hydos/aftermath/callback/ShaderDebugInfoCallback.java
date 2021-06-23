package me.hydos.aftermath.callback;

import org.jetbrains.annotations.Nullable;
import org.lwjgl.system.Callback;

import static org.lwjgl.system.MemoryUtil.NULL;


public abstract class ShaderDebugInfoCallback extends Callback implements ShaderDebugInfoCallbackI {
    public static ShaderDebugInfoCallback create(long functionPointer) {
        ShaderDebugInfoCallbackI instance = Callback.get(functionPointer);
        return instance instanceof ShaderDebugInfoCallback
                ? (ShaderDebugInfoCallback) instance
                : new Container(functionPointer, instance);
    }

    @Nullable
    public static ShaderDebugInfoCallback createSafe(long functionPointer) {
        return functionPointer == NULL ? null : create(functionPointer);
    }

    public static ShaderDebugInfoCallback create(ShaderDebugInfoCallbackI instance) {
        return instance instanceof ShaderDebugInfoCallback
                ? (ShaderDebugInfoCallback)instance
                : new Container(instance.address(), instance);
    }

    protected ShaderDebugInfoCallback() {
        super(CIF);
    }

    ShaderDebugInfoCallback(long functionPointer) {
        super(functionPointer);
    }

    private static final class Container extends ShaderDebugInfoCallback {
        private final ShaderDebugInfoCallbackI delegate;

        Container(long functionPointer, ShaderDebugInfoCallbackI delegate) {
            super(functionPointer);
            this.delegate = delegate;
        }

        @Override
        public void invoke(long pShaderDebugInfo, int shaderDebugInfoSize, long pUserData) {
            delegate.invoke(pShaderDebugInfo, shaderDebugInfoSize, pUserData);
        }
    }
}
