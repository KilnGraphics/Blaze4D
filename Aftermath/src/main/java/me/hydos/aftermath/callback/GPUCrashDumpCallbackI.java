package me.hydos.aftermath.callback;

import org.lwjgl.system.APIUtil;
import org.lwjgl.system.CallbackI;
import org.lwjgl.system.NativeType;
import org.lwjgl.system.libffi.FFICIF;
import org.lwjgl.system.libffi.LibFFI;

import java.io.IOException;

import static org.lwjgl.system.MemoryUtil.memGetAddress;
import static org.lwjgl.system.MemoryUtil.memGetInt;
import static org.lwjgl.system.libffi.LibFFI.*;

public interface GPUCrashDumpCallbackI extends CallbackI {
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
        try {
            invoke(
                    memGetAddress(memGetAddress(args)),
                    memGetInt(memGetAddress(args + POINTER_SIZE)),
                    memGetAddress(memGetAddress(args + 2 * POINTER_SIZE))
            );
        } catch (IOException e) {
            throw new RuntimeException("Failed to call back", e);
        }
    }

    void invoke(@NativeType("void *") long pGpuCrashDump, int gpuCrashDumpSize, @NativeType("void *") long pUserData) throws IOException;
}
