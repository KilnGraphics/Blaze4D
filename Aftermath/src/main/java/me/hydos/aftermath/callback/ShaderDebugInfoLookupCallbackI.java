package me.hydos.aftermath.callback;

import org.lwjgl.system.APIUtil;
import org.lwjgl.system.CallbackI;
import org.lwjgl.system.NativeType;
import org.lwjgl.system.libffi.FFICIF;
import org.lwjgl.system.libffi.LibFFI;

import static org.lwjgl.system.MemoryUtil.*;
import static org.lwjgl.system.libffi.LibFFI.*;
import static org.lwjgl.system.libffi.LibFFI.ffi_type_pointer;

public interface ShaderDebugInfoLookupCallbackI extends CallbackI {
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
                new long[]{
                        memGetLong(memGetAddress(args)),
                        memGetLong(memGetAddress(args + 1 * POINTER_SIZE))
                },
                memGetAddress(memGetAddress(args + 2 * POINTER_SIZE)),
                memGetAddress(memGetAddress(args + 3 * POINTER_SIZE))
        );
    }

    /**
     * @param pIdentifier        The length of the array is always 2
     * @param setShaderDebugInfo
     * @param pUserData
     */
    void invoke(@NativeType("GFSDK_Aftermath_ShaderDebugInfoIdentifier *") long[] pIdentifier, @NativeType("PFN_GFSDK_Aftermath_SetData") long setShaderDebugInfo, @NativeType("void *") long pUserData);
}
