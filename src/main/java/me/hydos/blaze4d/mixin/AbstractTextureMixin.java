package me.hydos.blaze4d.mixin;

import net.minecraft.client.texture.AbstractTexture;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(value = AbstractTexture.class, priority = 1001)
public class AbstractTextureMixin {

//    @Inject(method = "load", at = @At("TAIL"))
//    private void loadTexture(CallbackInfo ci) {
//        Blaze4D.rosella.getTextureManager().getOrLoadTexture(
//                Global.INSTANCE.fromBufferedImage(null, null),
//                Blaze4D.rosella,
//                VK10.VK_FORMAT_R8G8B8A8_SRGB
//        );
//    }

    @Inject(method = "bindTexture", at = @At("HEAD"), cancellable = true)
    private void nope(CallbackInfo ci) {
        ci.cancel();
    }

    @Inject(method = "getGlId", at = @At("HEAD"), cancellable = true)
    private void nope2(CallbackInfoReturnable<Integer> cir) {
        cir.setReturnValue(-1);
    }
}
