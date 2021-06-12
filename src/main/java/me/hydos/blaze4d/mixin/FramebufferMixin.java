package me.hydos.blaze4d.mixin;

import net.minecraft.client.gl.Framebuffer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(Framebuffer.class)
public class FramebufferMixin {

    @Inject(method = "resize", at = @At("HEAD"), cancellable = true)
    private void resizingBadAndWorst(int width, int height, boolean getError, CallbackInfo ci) {
        ci.cancel();
    }
}
