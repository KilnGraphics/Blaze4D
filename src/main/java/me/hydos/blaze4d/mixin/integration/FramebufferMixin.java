package me.hydos.blaze4d.mixin.integration;

import net.minecraft.client.gl.Framebuffer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(Framebuffer.class)
public class FramebufferMixin {

    @Shadow public int textureWidth;

    @Shadow public int textureHeight;

    @Shadow public int viewportWidth;

    @Shadow public int viewportHeight;

    @Inject(method = "resize", at = @At("HEAD"), cancellable = true)
    private void resizingBadAndWorst(int width, int height, boolean getError, CallbackInfo ci) {
        this.textureWidth = width;
        this.textureHeight = height;
        this.viewportWidth = width;
        this.viewportHeight = height;
        ci.cancel();
    }

    @Inject(method = "drawInternal", at = @At("HEAD"), cancellable = true)
    private void weDontSupportFbosAtm(int width, int height, boolean disableBlend, CallbackInfo ci) {
        ci.cancel();
    }
}
