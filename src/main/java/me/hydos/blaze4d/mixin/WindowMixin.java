package me.hydos.blaze4d.mixin;

import me.hydos.blaze4d.Blaze4D;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.render.io.Window;
import net.minecraft.client.WindowEventHandler;
import net.minecraft.client.WindowSettings;
import net.minecraft.client.util.MonitorTracker;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(net.minecraft.client.util.Window.class)
public class WindowMixin {

    @Inject(method = "<init>", at = @At("TAIL"))
    private void initializeRosellaWindow(WindowEventHandler eventHandler, MonitorTracker monitorTracker, WindowSettings settings, String videoMode, String title, CallbackInfo ci) {
        Blaze4D.window = new Window(title, settings.width, settings.height, true);
        Blaze4D.rosella = new Rosella("Blaze4D", true, Blaze4D.window);

//        Blaze4D.finishAndRender();
    }
}
