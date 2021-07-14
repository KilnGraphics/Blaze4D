package me.hydos.blaze4d.mixin.integration;

import net.minecraft.client.render.GameRenderer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(GameRenderer.class)
public class GameRendererMixin {

    @Inject(method = "updateWorldIcon()V", at = @At("HEAD"), cancellable = true)
    private void noUpdatingForNow(CallbackInfo ci) {
        ci.cancel();
    }
}
