package me.hydos.aftermath.callback;

import org.jetbrains.annotations.Nullable;
import org.lwjgl.system.Callback;

import java.nio.ByteBuffer;
import java.util.function.BiFunction;

import static org.lwjgl.system.MemoryUtil.NULL;


public abstract class ShaderDebugInfoLookupCallback extends Callback implements ShaderDebugInfoLookupCallbackI {
    public static ShaderDebugInfoLookupCallback create(long functionPointer) {
        ShaderDebugInfoLookupCallbackI instance = Callback.get(functionPointer);
        return instance instanceof ShaderDebugInfoLookupCallback
                ? (ShaderDebugInfoLookupCallback) instance
                : new Container(functionPointer, instance);
    }

    @Nullable
    public static ShaderDebugInfoLookupCallback createSafe(long functionPointer) {
        return functionPointer == NULL ? null : create(functionPointer);
    }

    public static ShaderDebugInfoLookupCallback create(ShaderDebugInfoLookupCallbackI instance) {
        return instance instanceof ShaderDebugInfoLookupCallback
                ? (ShaderDebugInfoLookupCallback)instance
                : new Container(instance.address(), instance);
    }

    protected ShaderDebugInfoLookupCallback() {
        super(CIF);
    }

    ShaderDebugInfoLookupCallback(long functionPointer) {
        super(functionPointer);
    }

    private static final class Container extends ShaderDebugInfoLookupCallback {
        private final ShaderDebugInfoLookupCallbackI delegate;

        Container(long functionPointer, ShaderDebugInfoLookupCallbackI delegate) {
            super(functionPointer);
            this.delegate = delegate;
        }


        @Override
        public void invoke(long[] pIdentifier, BiFunction<ByteBuffer, Integer, Integer> setShaderDebugInfo, long pUserData) {
            delegate.invoke(pIdentifier, setShaderDebugInfo, pUserData);
        }
    }
}
