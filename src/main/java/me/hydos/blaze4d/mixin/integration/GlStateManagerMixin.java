package me.hydos.blaze4d.mixin.integration;

import com.mojang.blaze3d.platform.GlStateManager;
import com.mojang.blaze3d.systems.RenderSystem;
import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(value = GlStateManager.class, remap = false)
public class GlStateManagerMixin {

    @Inject(method = {
            "_clearDepth",
            "_texParameter(IIF)V",
            "_texParameter(III)V",
            "_texImage2D",
            "_depthFunc",
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

    @Inject(method = "_bindTexture", at = @At("HEAD"), cancellable = true)
    private static void bindTexture(int texture, CallbackInfo ci) {
        GlobalRenderSystem.boundTextureId = texture;
        ci.cancel();
    }

    @Inject(method = "_genTexture", at = @At("HEAD"), cancellable = true)
    private static void genTexture(CallbackInfoReturnable<Integer> cir) {
        cir.setReturnValue(Blaze4D.rosella.getTextureManager().generateTextureId());
    }

    @Inject(method = "_deleteTexture", at = @At("HEAD"), cancellable = true)
    private static void deleteTexture(int texture, CallbackInfo ci) {
        Blaze4D.rosella.getTextureManager().deleteTexture(texture);
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