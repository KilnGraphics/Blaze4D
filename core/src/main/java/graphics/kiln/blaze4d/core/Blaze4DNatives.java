package graphics.kiln.blaze4d.core;

import jdk.incubator.foreign.*;

import java.lang.invoke.MethodHandle;
import java.util.Optional;

import static jdk.incubator.foreign.ValueLayout.*;

public class Blaze4DNatives {
    public static final SymbolLookup lookup;
    public static final CLinker linker;



    static {
        System.load(System.getProperty("b4d.native"));

        lookup = SymbolLookup.loaderLookup();
        linker = CLinker.systemCLinker();
    }

    private static MethodHandle lookupFunction(String name, FunctionDescriptor descriptor) {
        Optional<NativeSymbol> result = lookup.lookup(name);
        if (result.isPresent()) {
            return linker.downcallHandle(result.get(), descriptor);
        }
        throw new UnsatisfiedLinkError("Failed to find Blaze4D core function \"" + name + "\"");
    }
}
