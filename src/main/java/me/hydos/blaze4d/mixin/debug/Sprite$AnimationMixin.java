package me.hydos.blaze4d.mixin.debug;

import net.minecraft.client.renderer.texture.TextureAtlasSprite;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(TextureAtlasSprite.AnimatedTexture.class)
public class Sprite$AnimationMixin {

    @Inject(method = "tick", at = @At("HEAD"), cancellable = true)
    private void nope(CallbackInfo ci) {
        ci.cancel();
    }
}
