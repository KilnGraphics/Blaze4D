package me.hydos.aftermath.callback;

import org.jetbrains.annotations.Nullable;
import org.lwjgl.system.Callback;

import java.nio.ByteBuffer;
import java.util.function.BiFunction;

import static org.lwjgl.system.MemoryUtil.NULL;


public abstract class ShaderSourceDebugInfoLookupCallback extends Callback implements ShaderSourceDebugInfoLookupCallbackI {
    public static ShaderSourceDebugInfoLookupCallback create(long functionPointer) {
        ShaderSourceDebugInfoLookupCallbackI instance = Callback.get(functionPointer);
        return instance instanceof ShaderSourceDebugInfoLookupCallback
                ? (ShaderSourceDebugInfoLookupCallback) instance
                : new Container(functionPointer, instance);
    }

    @Nullable
    public static ShaderSourceDebugInfoLookupCallback createSafe(long functionPointer) {
        return functionPointer == NULL ? null : create(functionPointer);
    }

    public static ShaderSourceDebugInfoLookupCallback create(ShaderSourceDebugInfoLookupCallbackI instance) {
        return instance instanceof ShaderSourceDebugInfoLookupCallback
                ? (ShaderSourceDebugInfoLookupCallback)instance
                : new Container(instance.address(), instance);
    }

    protected ShaderSourceDebugInfoLookupCallback() {
        super(CIF);
    }

    ShaderSourceDebugInfoLookupCallback(long functionPointer) {
        super(functionPointer);
    }

    private static final class Container extends ShaderSourceDebugInfoLookupCallback {
        private final ShaderSourceDebugInfoLookupCallbackI delegate;

        Container(long functionPointer, ShaderSourceDebugInfoLookupCallbackI delegate) {
            super(functionPointer);
            this.delegate = delegate;
        }


        @Override
        public void invoke(String shaderDebugName, BiFunction<ByteBuffer, Integer, Integer> setShaderBinary, long pUserData) {
            delegate.invoke(shaderDebugName, setShaderBinary, pUserData);
        }
    }
}
