package me.hydos.blaze4d.mixin.integration;

import com.mojang.blaze3d.platform.GlStateManager;
import com.mojang.blaze3d.systems.RenderSystem;
import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import me.hydos.rosella.scene.object.impl.SimpleObjectManager;
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
        cir.setReturnValue(((SimpleObjectManager) Blaze4D.rosella.objectManager).textureManager.generateTextureId());
    }

    @Inject(method = "_deleteTexture", at = @At("HEAD"), cancellable = true)
    private static void deleteTexture(int texture, CallbackInfo ci) {
        ((SimpleObjectManager) Blaze4D.rosella.objectManager).textureManager.deleteTexture(texture);
        ci.cancel();
    }

    @Inject(method = "_genTextures", at = @At("HEAD"), cancellable = true)
    private static void genTextures(int[] is, CallbackInfo ci) {
        for (int i = 0; i < is.length; i++) {
            is[i] = ((SimpleObjectManager) Blaze4D.rosella.objectManager).textureManager.generateTextureId();
        }
        ci.cancel();
    }

    @Inject(method = "_deleteTextures", at = @At("HEAD"), cancellable = true)
    private static void deleteTextures(int[] is, CallbackInfo ci) {
        for (int textureId : is) {
            ((SimpleObjectManager) Blaze4D.rosella.objectManager).textureManager.deleteTexture(textureId);
        }
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
        Blaze4D.rosella.renderer.clearColor(red, green, blue, Blaze4D.rosella);
    }
}