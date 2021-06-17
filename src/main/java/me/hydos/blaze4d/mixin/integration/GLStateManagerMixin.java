package me.hydos.blaze4d.mixin.integration;

import com.mojang.blaze3d.platform.GlStateManager;
import me.hydos.blaze4d.Blaze4D;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(GlStateManager.class)
public class GLStateManagerMixin {

    @Inject(method = {
            "_clearColor",
            "_clearDepth",
            "_texParameter(IIF)V",
            "_texParameter(III)V",
            "_texImage2D",
            "_depthFunc",
            "_bindTexture",
            "_activeTexture",
            "_clear",
            "_disableScissorTest",
            "_enableScissorTest",
            "_disableDepthTest",
            "_enableDepthTest",
            "_enableCull",
            "_enableBlend",
            "_blendEquation",
            "_blendFunc",
            "_blendFuncSeparate",
            "_colorMask",
            "_depthMask",
            "_glBindFramebuffer"
    }, at = @At("HEAD"), cancellable = true)
    private static void clearColor(CallbackInfo ci) {
        //TODO: IMPL
        ci.cancel();
    }
}