package me.hydos.aftermath.callback;

import org.jetbrains.annotations.Nullable;
import org.lwjgl.system.Callback;

import java.nio.ByteBuffer;
import java.util.function.BiFunction;

import static org.lwjgl.system.MemoryUtil.NULL;


public abstract class ShaderLookupCallback extends Callback implements ShaderLookupCallbackI {
    public static ShaderLookupCallback create(long functionPointer) {
        ShaderLookupCallbackI instance = Callback.get(functionPointer);
        return instance instanceof ShaderLookupCallback
                ? (ShaderLookupCallback) instance
                : new Container(functionPointer, instance);
    }

    @Nullable
    public static ShaderLookupCallback createSafe(long functionPointer) {
        return functionPointer == NULL ? null : create(functionPointer);
    }

    public static ShaderLookupCallback create(ShaderLookupCallbackI instance) {
        return instance instanceof ShaderLookupCallback
                ? (ShaderLookupCallback)instance
                : new Container(instance.address(), instance);
    }

    protected ShaderLookupCallback() {
        super(CIF);
    }

    ShaderLookupCallback(long functionPointer) {
        super(functionPointer);
    }

    private static final class Container extends ShaderLookupCallback {
        private final ShaderLookupCallbackI delegate;

        Container(long functionPointer, ShaderLookupCallbackI delegate) {
            super(functionPointer);
            this.delegate = delegate;
        }


        @Override
        public void invoke(long shaderHash, BiFunction<ByteBuffer, Integer, Integer> setShaderBinary, long pUserData) {
            delegate.invoke(shaderHash, setShaderBinary, pUserData);
        }
    }
}
