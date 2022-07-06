package graphics.kiln.blaze4d.core.natives;

import jdk.incubator.foreign.SymbolLookup;
import org.apache.commons.lang3.SystemUtils;

import java.io.File;
import java.io.FileOutputStream;
import java.io.InputStream;
import java.io.OutputStream;

/**
 * Manages loading of the native library
 */
public class Lib {
    private static final String BASE_PATH = "natives";

    public static SymbolLookup nativeLookup = null;

    /**
     * Generates the resource name of the library for the current system.
     * This name can be passed to System.getResource() to retrieve the native binaries.
     *
     * This function does not validate that the binaries are actually bundled it only generates the name.
     *
     * @return The name of the library for the current system.
     */
    private static String getSystemLibName() {
        return BASE_PATH + "." + Os.getOs().name + "-" + Arch.getArch().name;
    }

    /**
     * Generates the File where the native library for the current system should be placed at.
     *
     * @return The File for the native library.
     */
    private static File getNativeFile() {
        return new File(System.getProperty("user.dir"), "b4d" + Os.getOs().generateLibName("b4d_core"));
    }

    /**
     * Ensures the native library is loaded and ready.
     *
     * It is safe to call this function multiple times and concurrently.
     */
    public static synchronized void prepareLib() {
        if (nativeLookup != null) {
            return;
        }

        String overwrite = System.getProperty("b4d.native");
        if (overwrite != null) {
            System.load(overwrite);
            nativeLookup = SymbolLookup.loaderLookup();
            return;
        }

        File natives = getNativeFile();
        if (!natives.mkdirs()) {
            throw new RuntimeException("Failed to create b4d natives directory");
        }
        if (natives.exists()) {
            if (!natives.delete()) {
                throw new RuntimeException("Failed to delete old b4d natives file");
            }
        }

        String resource = getSystemLibName();
        try (InputStream in = Lib.class.getResourceAsStream(resource)) {
            if (in == null) {
                throw new RuntimeException("Unsupported system configuration " + Os.getOs().name + "-" + Arch.getArch().name + ". Unable to find native library.");
            }

            try (OutputStream out = new FileOutputStream(natives)) {
                // TODO this is dumb should use a library
                byte[] bytes = in.readAllBytes();
                out.write(bytes);
            }
        } catch (Exception ex) {
            throw new RuntimeException("Failed to extract native library", ex);
        }

        System.load(natives.getAbsolutePath());
        nativeLookup = SymbolLookup.loaderLookup();
    }

    private enum Os {
        WINDOWS("windows"),
        GENERIC_LINUX("generic_linux"),
        MAC("mac");

        public final String name;

        Os(String name) {
            this.name = name;
        }

        String generateLibName(String name) {
            return switch (this) {
                case WINDOWS -> name + ".dll";
                case GENERIC_LINUX, MAC -> "lib" + name + ".so";
            };
        }

        static Os getOs() {
            if (SystemUtils.IS_OS_WINDOWS) {
                return WINDOWS;
            } else if(SystemUtils.IS_OS_LINUX) {
                return GENERIC_LINUX;
            } else if(SystemUtils.IS_OS_MAC) {
                return MAC;
            } else {
                throw new UnsupportedOperationException("Unknown os: " + SystemUtils.OS_NAME);
            }
        }
    }

    private enum Arch {
        I686("i686"),
        AMD64("amd64"),
        AARCH64("aarch64");

        public final String name;

        Arch(String name) {
            this.name = name;
        }

        static Arch getArch() {
            return switch (SystemUtils.OS_ARCH) {
                case "x86" -> I686;
                case "amd64" -> AMD64;
                case "aarch64" -> AARCH64;
                default -> throw new UnsupportedOperationException("Unknown arch: " + SystemUtils.OS_ARCH);
            };
        }
    }
}