package me.hydos.blaze4d.mixin.blaze3d.platform;

import com.mojang.blaze3d.platform.TextureUtil;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(TextureUtil.class)
public class TextureUtilMixin {

    @Inject(method = "generateTextureId", at = @At("HEAD"), cancellable = true)
    private static void cancelled_generateTextureId(CallbackInfoReturnable<Integer> cir) {
        cir.setReturnValue(-1);
    }
}
