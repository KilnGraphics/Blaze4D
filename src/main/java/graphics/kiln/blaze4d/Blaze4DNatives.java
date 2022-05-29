package graphics.kiln.blaze4d;

import jdk.incubator.foreign.*;

import java.lang.invoke.MethodHandle;
import java.util.Optional;

public class Blaze4DNatives {

    private static SymbolLookup lookup;
    private static CLinker linker;

    public static MethodHandle b4dEmulatorVertexFormatSetBuilderNew;
    public static MethodHandle b4dEmulatorVertexFormatSetBuilderDestroy;
    public static MethodHandle b4dEmulatorVertexFormatSetBuilderAddFormat;

    public static MethodHandle b4dInit;
    public static MethodHandle b4dDestroy;
    public static MethodHandle b4dSetVertexFormats;
    public static MethodHandle b4dCreateStaticMesh;
    public static MethodHandle b4dDestroyStaticMesh;
    public static MethodHandle b4dStartFrame;

    public static MethodHandle b4dPassSetModelViewMatrix;
    public static MethodHandle b4dPassSetProjectionMatrix;
    public static MethodHandle b4dPassDrawStatic;
    public static MethodHandle b4dPassDrawImmediate;
    public static MethodHandle b4dEndFrame;

    public static SequenceLayout meshDataLayout;
    public static SequenceLayout b4dVertexFormatLayout;

    static void load() {
        lookup = SymbolLookup.loaderLookup();
        linker = CLinker.systemCLinker();

        b4dEmulatorVertexFormatSetBuilderNew = lookupFunction("b4d_emulator_vertex_format_set_builder_new",
                FunctionDescriptor.of(ValueLayout.ADDRESS)
        );

        b4dEmulatorVertexFormatSetBuilderDestroy = lookupFunction("b4d_emulator_vertex_format_set_builder_destroy",
                FunctionDescriptor.ofVoid(ValueLayout.ADDRESS)
        );

        b4dEmulatorVertexFormatSetBuilderAddFormat = lookupFunction("b4d_emulator_vertex_format_builder_add_format",
                FunctionDescriptor.ofVoid(ValueLayout.ADDRESS, ValueLayout.JAVA_INT)
        );

        b4dInit = lookupFunction("b4d_init",
                FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.ADDRESS, ValueLayout.ADDRESS, ValueLayout.JAVA_INT)
        );

        b4dDestroy = lookupFunction("b4d_destroy",
                FunctionDescriptor.ofVoid(ValueLayout.ADDRESS)
        );

        b4dSetVertexFormats = lookupFunction("b4d_set_vertex_formats",
                FunctionDescriptor.ofVoid(ValueLayout.ADDRESS, ValueLayout.ADDRESS, ValueLayout.JAVA_INT)
        );

        b4dCreateStaticMesh = lookupFunction("b4d_create_static_mesh",
                FunctionDescriptor.of(ValueLayout.JAVA_LONG, ValueLayout.ADDRESS, ValueLayout.ADDRESS)
        );

        b4dDestroyStaticMesh = lookupFunction("b4d_destroy_static_mesh",
                FunctionDescriptor.ofVoid(ValueLayout.ADDRESS, ValueLayout.JAVA_LONG)
        );

        b4dStartFrame = lookupFunction("b4d_start_frame",
                FunctionDescriptor.of(ValueLayout.ADDRESS, ValueLayout.ADDRESS, ValueLayout.JAVA_INT, ValueLayout.JAVA_INT)
        );
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
