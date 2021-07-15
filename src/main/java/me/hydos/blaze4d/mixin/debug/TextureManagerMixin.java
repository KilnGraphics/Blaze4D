package me.hydos.blaze4d.mixin.debug;

import net.minecraft.client.renderer.texture.TextureManager;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(TextureManager.class)
public class TextureManagerMixin {

    @Inject(method = "tick", at = @At("HEAD"), cancellable = true)
    private void e(CallbackInfo ci) {
        ci.cancel();
    }
}
