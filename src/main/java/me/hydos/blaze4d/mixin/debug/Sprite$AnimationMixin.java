package me.hydos.blaze4d.mixin.debug;

import net.minecraft.client.texture.Sprite;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(Sprite.Animation.class)
public class Sprite$AnimationMixin {

    @Inject(method = "tick", at = @At("HEAD"), cancellable = true)
    private void nope(CallbackInfo ci) {
        ci.cancel();
    }
}
