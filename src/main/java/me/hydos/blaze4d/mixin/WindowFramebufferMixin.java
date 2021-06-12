package me.hydos.blaze4d.mixin;

import net.minecraft.client.gl.Framebuffer;
import net.minecraft.client.gl.WindowFramebuffer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(WindowFramebuffer.class)
public class WindowFramebufferMixin extends Framebuffer {

    public WindowFramebufferMixin(boolean useDepth) {
        super(useDepth);
    }

    @Inject(method = "supportColor", at = @At("HEAD"), cancellable = true)
    private void supportAllColours(CallbackInfoReturnable<Boolean> cir) {
        cir.setReturnValue(true);
    }

    @Inject(method = "supportsDepth", at = @At("HEAD"), cancellable = true)
    private void supportDepth(CallbackInfoReturnable<Boolean> cir) {
        cir.setReturnValue(true);
    }

    @Inject(method = "initSize", at = @At("HEAD"), cancellable = true)
    private void fbosAreBad(int width, int height, CallbackInfo ci) {
        this.viewportWidth = 1920;
        this.viewportHeight = 1080;
        this.textureWidth = 1920;
        this.textureHeight = 1080;
        ci.cancel();
    }
}
