package me.hydos.blaze4d.mixin.testing.screens;

import net.minecraft.client.gui.RotatingCubeMapRenderer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(RotatingCubeMapRenderer.class)
public class RotatingCubeMapRendererMixin {

/*    @Inject(method = "render", at = @At("HEAD"), cancellable = true)
    private void dontRenderPanoramaplsktnx(float delta, float alpha, CallbackInfo ci) {
        ci.cancel();
    }*/
}
