package graphics.kiln.blaze4d.mixin.integration;

import com.mojang.blaze3d.platform.GlDebug;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(GlDebug.class)
public class GlDebugMixin {

    /*
    @Inject(method = "enableDebugCallback", at = @At("HEAD"), cancellable = true)
    private static void debuggingIsForTheWeak(int verbosity, boolean sync, CallbackInfo ci) {
        ci.cancel();
    }
    */
}
