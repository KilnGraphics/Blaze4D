package me.hydos.blaze4d;

import net.fabricmc.loader.api.entrypoint.PreLaunchEntrypoint;
import org.lwjgl.system.Configuration;
import org.lwjgl.system.Platform;
import org.lwjgl.system.jemalloc.JEmalloc;

public class Blaze4DPreLaunch implements PreLaunchEntrypoint {
    public static final boolean DEBUG_MEMORY_ENABLED = false;
    public static final int LWJGL_STACK_SIZE = 2048; // 2mb instead of default 64kb. TODO: don't rely on a larger stack size

    @Override
    public void onPreLaunch() {
        Configuration.DEBUG_MEMORY_ALLOCATOR.set(DEBUG_MEMORY_ENABLED);
        Configuration.STACK_SIZE.set(LWJGL_STACK_SIZE);

        // jemalloc has a memory leak bug on windows from 5.0.0 to 5.2.0
        if (Platform.get().equals(Platform.WINDOWS) && JEmalloc.JEMALLOC_VERSION_MAJOR == 5 &&
                (JEmalloc.JEMALLOC_VERSION_MINOR >= 0 && JEmalloc.JEMALLOC_VERSION_MINOR < 2) || (JEmalloc.JEMALLOC_VERSION_MINOR == 2 && JEmalloc.JEMALLOC_VERSION_BUGFIX < 1)) {
            Configuration.MEMORY_ALLOCATOR.set("system");
        }
    }
}
