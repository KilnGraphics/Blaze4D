package graphics.kiln.blaze4d;

import jdk.incubator.foreign.*;

import java.lang.invoke.MethodHandle;
import java.util.Optional;

import static jdk.incubator.foreign.ValueLayout.*;

public class Blaze4DNatives {

    public static SymbolLookup lookup;
    public static CLinker linker;

    public static GroupLayout meshDataLayout;
    public static GroupLayout vertexFormatLayout;

    public static MethodHandle b4dInitExternalLoggerHandle;
    public static MethodHandle b4dPreInitGlfwHandle;
    public static MethodHandle b4dCreateGlfwSurfaceProviderHandle;

    public static MethodHandle b4dInitHandle;
    public static MethodHandle b4dDestroyHandle;
    public static MethodHandle b4dSetVertexFormatsHandle;
    public static MethodHandle b4dCreateStaticMeshHandle;
    public static MethodHandle b4dDestroyStaticMeshHandle;
    public static MethodHandle b4dStartFrameHandle;

    public static MethodHandle b4dPassSetModelViewMatrixHandle;
    public static MethodHandle b4dPassSetProjectionMatrixHandle;
    public static MethodHandle b4dPassDrawStaticHandle;
    public static MethodHandle b4dPassDrawImmediateHandle;
    public static MethodHandle b4dEndFrameHandle;

    static void load() {
        System.load(System.getProperty("b4d.native"));

        lookup = SymbolLookup.loaderLookup();
        linker = CLinker.systemCLinker();

        meshDataLayout = MemoryLayout.structLayout(
                ADDRESS,
                JAVA_LONG,
                ADDRESS,
                JAVA_LONG,
                JAVA_INT,
                JAVA_INT
        );

        vertexFormatLayout = MemoryLayout.structLayout(
                JAVA_INT,
                JAVA_INT,
                JAVA_INT,
                JAVA_INT
        );

        b4dInitExternalLoggerHandle = lookupFunction("b4d_init_external_logger",
                FunctionDescriptor.ofVoid(ADDRESS)
        );

        b4dPreInitGlfwHandle = lookupFunction("b4d_pre_init_glfw",
                FunctionDescriptor.ofVoid(ADDRESS)
        );

        b4dCreateGlfwSurfaceProviderHandle = lookupFunction("b4d_create_glfw_surface_provider",
                FunctionDescriptor.of(ADDRESS, ADDRESS, ADDRESS, ADDRESS)
        );

        b4dInitHandle = lookupFunction("b4d_init",
                FunctionDescriptor.of(ADDRESS, ADDRESS, JAVA_INT)
        );

        b4dDestroyHandle = lookupFunction("b4d_destroy",
                FunctionDescriptor.ofVoid(ADDRESS)
        );

        b4dSetVertexFormatsHandle = lookupFunction("b4d_set_vertex_formats",
                FunctionDescriptor.ofVoid(ADDRESS, ADDRESS, JAVA_INT)
        );

        b4dCreateStaticMeshHandle = lookupFunction("b4d_create_static_mesh",
                FunctionDescriptor.of(JAVA_LONG, ADDRESS, ADDRESS)
        );

        b4dDestroyStaticMeshHandle = lookupFunction("b4d_destroy_static_mesh",
                FunctionDescriptor.ofVoid(ADDRESS, JAVA_LONG)
        );

        b4dStartFrameHandle = lookupFunction("b4d_start_frame",
                FunctionDescriptor.of(ADDRESS, ADDRESS, JAVA_INT, JAVA_INT)
        );

        b4dPassSetModelViewMatrixHandle = lookupFunction("b4d_pass_set_model_view_matrix",
                FunctionDescriptor.ofVoid(ADDRESS, ADDRESS)
        );

        b4dPassSetProjectionMatrixHandle = lookupFunction("b4d_pass_set_projection_matrix",
                FunctionDescriptor.ofVoid(ADDRESS, ADDRESS)
        );

        b4dPassDrawStaticHandle = lookupFunction("b4d_pass_draw_static",
                FunctionDescriptor.ofVoid(ADDRESS, JAVA_LONG, JAVA_INT)
        );

        b4dPassDrawImmediateHandle = lookupFunction("b4d_pass_draw_immediate",
                FunctionDescriptor.ofVoid(ADDRESS, ADDRESS, JAVA_INT)
        );

        b4dEndFrameHandle = lookupFunction("b4d_end_frame",
                FunctionDescriptor.ofVoid(ADDRESS)
        );
    }

    public static void b4dInitExternalLogger(NativeSymbol logFn) {
        try {
            b4dInitExternalLoggerHandle.invoke(logFn);
        } catch (Throwable e) {
            throw new RuntimeException(e);
        }
    }

    public static void b4dPreInitGlfw(MemoryAddress pfnGlfwInitVulkanLoader) {
        try {
            b4dPreInitGlfwHandle.invoke(pfnGlfwInitVulkanLoader);
        } catch (Throwable e) {
            throw new RuntimeException(e);
        }
    }

    public static MemoryAddress b4dCreateGlfwSurfaceProvider(long window, MemoryAddress pfnGlfwGetRequiredInstanceExtensions, MemoryAddress pfnGlfwCreateWindowSurface) {
        try {
            return (MemoryAddress) b4dCreateGlfwSurfaceProviderHandle.invoke(MemoryAddress.ofLong(window), pfnGlfwGetRequiredInstanceExtensions, pfnGlfwCreateWindowSurface);
        } catch (Throwable e) {
            throw new RuntimeException(e);
        }
    }

    public static MemoryAddress b4dInit(MemoryAddress surface, boolean enableValidation) {
        int validation = enableValidation ? 1 : 0;
        try {
            return (MemoryAddress) b4dInitHandle.invoke(surface, validation);
        } catch (Throwable e) {
            throw new RuntimeException(e);
        }
    }

    public static void b4dDestroy(MemoryAddress b4d) {
        try {
            b4dDestroyHandle.invoke(b4d);
        } catch (Throwable e) {
            throw new RuntimeException(e);
        }
    }

    public static void b4dSetVertexFormats(MemoryAddress b4d, MemoryAddress vertexFormats, int formatCount) {
        try {
            b4dSetVertexFormatsHandle.invoke(b4d, vertexFormats, formatCount);
        } catch (Throwable e) {
            throw new RuntimeException(e);
        }
    }

    public static long b4dCreateStaticMesh(MemoryAddress b4d, MemoryAddress meshData) {
        try {
            return (long) b4dCreateStaticMeshHandle.invoke(b4d, meshData);
        } catch (Throwable e) {
            throw new RuntimeException(e);
        }
    }

    public static void b4dDestroyStaticMesh(MemoryAddress b4d, long meshId) {
        try {
            b4dDestroyStaticMeshHandle.invoke(b4d, meshId);
        } catch (Throwable e) {
            throw new RuntimeException(e);
        }
    }

    public static MemoryAddress b4dStartFrame(MemoryAddress b4d, int windowWidth, int windowHeight) {
        try {
            return (MemoryAddress) b4dStartFrameHandle.invoke(b4d, windowWidth, windowHeight);
        } catch (Throwable e) {
            throw new RuntimeException(e);
        }
    }

    public static void b4dPassSetModelViewMatrix(MemoryAddress pass, MemoryAddress matrix) {
        try {
            b4dPassSetModelViewMatrixHandle.invoke(pass, matrix);
        } catch (Throwable e) {
            throw new RuntimeException(e);
        }
    }

    public static void b4dPassSetProjectionMatrix(MemoryAddress pass, MemoryAddress matrix) {
        try {
            b4dPassSetProjectionMatrixHandle.invoke(pass, matrix);
        } catch (Throwable e) {
            throw new RuntimeException(e);
        }
    }

    public static void b4dPassDrawStatic(MemoryAddress pass, long meshId, int typeId) {
        try {
            b4dPassDrawStaticHandle.invoke(pass, meshId, typeId);
        } catch (Throwable e) {
            throw new RuntimeException(e);
        }
    }

    public static void b4dPassDrawImmediate(MemoryAddress pass, MemoryAddress meshData, int typeId) {
        try {
            b4dPassDrawImmediateHandle.invoke(pass, meshData, typeId);
        } catch (Throwable e) {
            throw new RuntimeException(e);
        }
    }

    public static void b4dEndFrame(MemoryAddress pass) {
        try {
            b4dEndFrameHandle.invoke(pass);
        } catch (Throwable e) {
            throw new RuntimeException(e);
        }
    }

    private static MethodHandle lookupFunction(String name, FunctionDescriptor descriptor) {
        Optional<NativeSymbol> result = lookup.lookup(name);
        if (result.isPresent()) {
            return linker.downcallHandle(result.get(), descriptor);
        }
        Blaze4D.LOGGER.fatal("Failed to find Blaze4D core function \"" + name + "\"");
        throw new RuntimeException("Failed to find Blaze4D core function \"" + name + "\"");
    }
}
