package me.hydos.blaze4d;

import net.fabricmc.loader.api.entrypoint.PreLaunchEntrypoint;
import org.lwjgl.system.Configuration;

public class Blaze4DPreLaunch implements PreLaunchEntrypoint {
    public static final int LWJGL_STACK_SIZE = 4096; // 4mb instead of default 64kb. TODO: don't rely on a larger stack size

    @Override
    public void onPreLaunch() {
        Configuration.STACK_SIZE.set(LWJGL_STACK_SIZE);
    }
}
