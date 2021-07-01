package me.hydos.blaze4d.mixin.integration;

import com.mojang.blaze3d.platform.GlStateManager;
import com.mojang.blaze3d.systems.RenderSystem;
import me.hydos.blaze4d.Blaze4D;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(value = GlStateManager.class, remap = false)
public class GlStateManagerMixin {

    @Inject(method = {
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
    private static void unimplementedGlCalls(CallbackInfo ci) {
        //TODO: IMPL
        ci.cancel();
    }

    /**
     * @author Blaze4D
     * @reason Clear Color Integration
     * <p>
     * Minecraft may be regarded as having bad code, but sometimes its ok.
     */
    @Overwrite
    public static void _clearColor(float red, float green, float blue, float alpha) {
        RenderSystem.assertThread(RenderSystem::isOnRenderThreadOrInit);
        Blaze4D.rosella.getRenderer().clearColor(red, green, blue, Blaze4D.rosella);
    }
}