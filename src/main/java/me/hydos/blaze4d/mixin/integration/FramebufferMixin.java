package me.hydos.blaze4d.mixin.integration;

import com.mojang.blaze3d.pipeline.RenderTarget;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(RenderTarget.class)
public class FramebufferMixin {

    @Shadow
    public int width;

    @Shadow
    public int height;

    @Shadow
    public int viewWidth;

    @Shadow
    public int viewHeight;

    @Inject(method = "resize", at = @At("HEAD"), cancellable = true)
    private void resizingBadAndWorst(int width, int height, boolean getError, CallbackInfo ci) {
        this.width = width;
        this.height = height;
        this.viewWidth = width;
        this.viewHeight = height;
        ci.cancel();
    }

    @Inject(method = "_blitToScreen", at = @At("HEAD"), cancellable = true)
    private void weDontSupportFbosAtm(int width, int height, boolean disableBlend, CallbackInfo ci) {
        ci.cancel();
    }

    @Inject(method = "clear", at = @At("HEAD"), cancellable = true)
    private void thisMessesUpSkyColor(boolean clearError, CallbackInfo ci) {
        ci.cancel();
    }
}
