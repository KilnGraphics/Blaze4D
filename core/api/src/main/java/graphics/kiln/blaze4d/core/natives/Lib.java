package graphics.kiln.blaze4d.core.natives;

import graphics.kiln.blaze4d.core.Blaze4DCore;
import jdk.incubator.foreign.SymbolLookup;

import com.google.gson.Gson;
import org.apache.commons.lang3.SystemUtils;

import java.io.*;
import java.nio.charset.StandardCharsets;

/**
 * Manages loading of the native library
 */
public class Lib {
    public static SymbolLookup nativeLookup = null;

    /**
     * Attempts to load the b4d core natives.
     *
     * It is safe to call this function multiple times and concurrently.
     */
    public static synchronized void loadNatives() {
        if (nativeLookup != null) {
            return;
        }

        String overwrite = System.getProperty("b4d_core.native_lib");
        if (overwrite != null) {
            System.load(overwrite);
            nativeLookup = SymbolLookup.loaderLookup();
            return;
        }

        NativeLib nativeLib = NativeLib.loadSystemLibInfo();
        if (nativeLib == null) {
            throw new UnsupportedOperationException("Unable to find natives for current system. Os: " + Os.getOs().name + " Arch: " + Arch.getArch().name);
        }
        Blaze4DCore.LOGGER.info("Found natives for current system. Os: " + nativeLib.os.name + " Arch: " + nativeLib.arch.name + " Version: " + nativeLib.version);

        File nativeFile = nativeLib.extractNatives(new File(System.getProperty("user.dir"), "b4d" + File.separator + "natives"));

        System.load(nativeFile.getAbsolutePath());
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

        static Os getOsForString(String name) {
            return switch (name) {
                case "windows" -> WINDOWS;
                case "generic_linux" -> GENERIC_LINUX;
                case "mac" -> MAC;
                default -> throw new IllegalArgumentException("Invalid os name: " + name);
            };
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
            return getArchForString(SystemUtils.OS_ARCH);
        }

        static Arch getArchForString(String name) {
            return switch (name) {
                case "x86" -> I686;
                case "amd64" -> AMD64;
                case "aarch64" -> AARCH64;
                default -> throw new UnsupportedOperationException("Unknown arch: " + SystemUtils.OS_ARCH);
            };
        }
    }

    private record NativeLib(Os os, Arch arch, Version version, String libResourcePath) {
        private static final String NATIVES_DESCRIPTION_PATH = "natives.json";

        /**
         * @return A NativeLib instance describing the native that should be used for the current system configuration.
         *         If the current system is unsupported null is returned.
         */
        static NativeLib loadSystemLibInfo() {
            NativeLibJson[] natives;
            try (InputStream in = NativeLib.class.getResourceAsStream(NATIVES_DESCRIPTION_PATH)) {
                if (in == null) {
                    throw new RuntimeException("Failed to load natives description. Resource does not exist " + NATIVES_DESCRIPTION_PATH);
                }
                BufferedReader reader = new BufferedReader(new InputStreamReader(in, StandardCharsets.UTF_8));

                natives = new Gson().fromJson(reader, NativeLibJson[].class);
            } catch (IOException e) {
                throw new RuntimeException("Failed to load natives description.", e);
            }

            Os sysOs = Os.getOs();
            Arch sysArch = Arch.getArch();
            for (NativeLibJson nativeJson : natives) {
                NativeLib nativeLib = nativeJson.toNativeLib();

                if (sysOs == nativeLib.os() && sysArch == nativeLib.arch()) {
                    return nativeLib;
                }
            }

            return null;
        }

        File extractNatives(File dstDirectory) {
            if (!dstDirectory.exists()) {
                if (!dstDirectory.mkdirs()) {
                    throw new RuntimeException("Failed to make natives directory");
                }
            }
            if (!dstDirectory.isDirectory()) {
                throw new RuntimeException("Natives directory is not a directory");
            }

            String fileName = System.mapLibraryName("b4d_core-" + this.version);
            File nativesFile = new File(dstDirectory, fileName);

            if(!nativesFile.isFile()) {
                if(nativesFile.exists()) {
                    throw new RuntimeException("Natives file already exists but is not a file");
                }

                this.copyToFile(nativesFile);
            }

            return nativesFile;
        }

        private void copyToFile(File dst) {
            try (InputStream in = NativeLib.class.getResourceAsStream(this.libResourcePath)) {
                if (in == null) {
                    throw new RuntimeException("Invalid native lib resource path: " + this.libResourcePath);
                }

                try (OutputStream out = new FileOutputStream(dst)) {
                    byte[] buffer = new byte[1024 * 1024 * 16];
                    int readBytes;

                    while((readBytes = in.read(buffer)) != -1) {
                        out.write(buffer, 0, readBytes);
                    }
                }
            } catch (IOException e) {
                throw new RuntimeException("Failed to extract native lib.", e);
            }
        }

        record Version(int major, int minor, int patch, long buildId) {
            @Override
            public String toString() {
                return major + "." + minor + "." + patch + "_" + buildId;
            }
        }

        private static class NativeLibJson {
            String os;
            String arch;
            int versionMajor;
            int versionMinor;
            int versionPatch;
            long buildId;
            String libResourcePath;

            NativeLib toNativeLib() {
                return new NativeLib(
                        Os.getOsForString(this.os),
                        Arch.getArchForString(this.arch),
                        new Version(this.versionMajor, this.versionMinor, this.versionPatch, this.buildId),
                        libResourcePath
                );
            }
        }
    }
}