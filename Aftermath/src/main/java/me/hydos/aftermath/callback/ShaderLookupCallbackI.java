package me.hydos.aftermath.callback;

import org.lwjgl.system.APIUtil;
import org.lwjgl.system.CallbackI;
import org.lwjgl.system.NativeType;
import org.lwjgl.system.libffi.FFICIF;
import org.lwjgl.system.libffi.LibFFI;

import static org.lwjgl.system.MemoryUtil.memGetAddress;
import static org.lwjgl.system.MemoryUtil.memGetLong;
import static org.lwjgl.system.libffi.LibFFI.*;

public interface ShaderLookupCallbackI extends CallbackI {
    FFICIF CIF = APIUtil.apiCreateCIF(
            LibFFI.FFI_DEFAULT_ABI,
            ffi_type_void,
            ffi_type_pointer, ffi_type_uint32, ffi_type_pointer
    );

    @Override
    default FFICIF getCallInterface() {
        return CIF;
    }

    @Override
    default void callback(long ret, long args) {
        invoke(
                memGetLong(memGetAddress(args)),
                memGetLong(memGetAddress(args + 1 * POINTER_SIZE)),
                memGetAddress(memGetAddress(args + 2 * POINTER_SIZE))
        );
    }

    void invoke(@NativeType("GFSDK_Aftermath_ShaderHash *") long pShaderHash, @NativeType("PFN_GFSDK_Aftermath_SetData") long setShaderBinary, @NativeType("void *") long pUserData);
}
