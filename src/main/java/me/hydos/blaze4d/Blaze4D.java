package me.hydos.blaze4d;

import me.hydos.blaze4d.api.Materials;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.render.io.Window;
import net.fabricmc.api.ModInitializer;

public class Blaze4D implements ModInitializer {

    public static Rosella rosella;
    public static Window window;

    public static void finishAndRender() {
        rosella.getRenderer().rebuildCommandBuffers(rosella.getRenderer().renderPass, rosella);
        window.onMainLoop(() -> rosella.getRenderer().render(rosella));
    }

    public static void prepare() {
        Materials.initialize(rosella);
    }

    @Override
    public void onInitialize() {
        System.loadLibrary("renderdoc");
    }
}
