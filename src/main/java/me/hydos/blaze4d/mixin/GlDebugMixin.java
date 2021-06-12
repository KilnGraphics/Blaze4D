package me.hydos.blaze4d.mixin;

import net.minecraft.client.gl.GlDebug;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(GlDebug.class)
public class GlDebugMixin {

    @Inject(method = "enableDebug", at = @At("HEAD"), cancellable = true)
    private static void debuggingIsForTheWeak(int verbosity, boolean sync, CallbackInfo ci) {
        ci.cancel();
    }
}
