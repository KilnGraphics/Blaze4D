package me.hydos.aftermath.callback;

import org.jetbrains.annotations.Nullable;
import org.lwjgl.system.Callback;

import java.nio.ByteBuffer;
import java.util.function.BiFunction;

import static org.lwjgl.system.MemoryUtil.NULL;


public abstract class ShaderInstructionsLookupCallback extends Callback implements ShaderInstructionsLookupCallbackI {
    public static ShaderInstructionsLookupCallback create(long functionPointer) {
        ShaderInstructionsLookupCallbackI instance = Callback.get(functionPointer);
        return instance instanceof ShaderInstructionsLookupCallback
                ? (ShaderInstructionsLookupCallback) instance
                : new Container(functionPointer, instance);
    }

    @Nullable
    public static ShaderInstructionsLookupCallback createSafe(long functionPointer) {
        return functionPointer == NULL ? null : create(functionPointer);
    }

    public static ShaderInstructionsLookupCallback create(ShaderInstructionsLookupCallbackI instance) {
        return instance instanceof ShaderInstructionsLookupCallback
                ? (ShaderInstructionsLookupCallback)instance
                : new Container(instance.address(), instance);
    }

    protected ShaderInstructionsLookupCallback() {
        super(CIF);
    }

    ShaderInstructionsLookupCallback(long functionPointer) {
        super(functionPointer);
    }

    private static final class Container extends ShaderInstructionsLookupCallback {
        private final ShaderInstructionsLookupCallbackI delegate;

        Container(long functionPointer, ShaderInstructionsLookupCallbackI delegate) {
            super(functionPointer);
            this.delegate = delegate;
        }


        @Override
        public void invoke(long pShaderInstructionsHash, BiFunction<ByteBuffer, Integer, Integer> setShaderBinary, long pUserData) {
            delegate.invoke(pShaderInstructionsHash, setShaderBinary, pUserData);
        }
    }
}
