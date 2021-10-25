package graphics.kiln.blaze4d.mixin.integration;

import com.mojang.blaze3d.pipeline.MainTarget;
import com.mojang.blaze3d.pipeline.RenderTarget;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(MainTarget.class)
public class WindowFramebufferMixin extends RenderTarget {

    public WindowFramebufferMixin(boolean useDepth) {
        super(useDepth);
    }

    @Inject(method = "allocateColorAttachment", at = @At("HEAD"), cancellable = true)
    private void supportAllColours(CallbackInfoReturnable<Boolean> cir) {
        cir.setReturnValue(true);
    }

    @Inject(method = "allocateDepthAttachment", at = @At("HEAD"), cancellable = true)
    private void supportDepth(CallbackInfoReturnable<Boolean> cir) {
        cir.setReturnValue(true);
    }

    @Inject(method = "createFrameBuffer", at = @At("HEAD"), cancellable = true)
    private void fbosAreBad(int width, int height, CallbackInfo ci) {
        this.viewWidth = width;
        this.viewHeight = height;
        this.width = width;
        this.height = height;
        ci.cancel();
    }
}
