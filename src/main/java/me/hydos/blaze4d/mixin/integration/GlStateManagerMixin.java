package me.hydos.blaze4d.mixin.integration;

import com.mojang.blaze3d.platform.GlStateManager;
import com.mojang.blaze3d.systems.RenderSystem;
import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import me.hydos.blaze4d.api.util.GlConversions;
import me.hydos.rosella.scene.object.impl.SimpleObjectManager;
import me.hydos.rosella.util.Color;
import org.lwjgl.opengl.GL13;
import org.lwjgl.vulkan.VK10;
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
            "_clear",
            "_glBindFramebuffer"
    }, at = @At("HEAD"), cancellable = true)
    private static void unimplementedGlCalls(CallbackInfo ci) {
        //TODO: IMPL
        ci.cancel();
    }

    @Inject(method = "_enableColorLogicOp", at = @At("HEAD"), cancellable = true)
    private static void enableColorLogicOp(CallbackInfo ci) {
        GlobalRenderSystem.currentStateInfo.setColorLogicOpEnabled(true);
        ci.cancel();
    }

    @Inject(method = "_disableColorLogicOp", at = @At("HEAD"), cancellable = true)
    private static void disableColorLogicOp(CallbackInfo ci) {
        GlobalRenderSystem.currentStateInfo.setColorLogicOpEnabled(false);
        ci.cancel();
    }

    @Inject(method = "_logicOp", at = @At("HEAD"), cancellable = true)
    private static void logicOp(int op, CallbackInfo ci) {
        GlobalRenderSystem.currentStateInfo.setColorLogicOp(GlConversions.glToVkLogicOp(op));
        ci.cancel();
    }

    @Inject(method = "_enableDepthTest", at = @At("HEAD"), cancellable = true)
    private static void enableDepthTest(CallbackInfo ci) {
        GlobalRenderSystem.currentStateInfo.setDepthTestEnabled(true);
        ci.cancel();
    }

    @Inject(method = "_disableDepthTest", at = @At("HEAD"), cancellable = true)
    private static void disableDepthTest(CallbackInfo ci) {
        GlobalRenderSystem.currentStateInfo.setDepthTestEnabled(false);
        ci.cancel();
    }

    @Inject(method = "_enableScissorTest", at = @At("HEAD"), cancellable = true)
    private static void enableScissorTest(CallbackInfo ci) {
        GlobalRenderSystem.currentStateInfo.setScissorEnabled(true);
        ci.cancel();
    }

    @Inject(method = "_disableScissorTest", at = @At("HEAD"), cancellable = true)
    private static void disableScissorTest(CallbackInfo ci) {
        GlobalRenderSystem.currentStateInfo.setScissorEnabled(false);
        ci.cancel();
    }

    @Inject(method = "_scissorBox", at = @At("HEAD"), cancellable = true)
    private static void scissorBox(int x, int y, int width, int height, CallbackInfo ci) {
        GlobalRenderSystem.currentStateInfo.setScissorX(x);
        GlobalRenderSystem.currentStateInfo.setScissorY(y);
        GlobalRenderSystem.currentStateInfo.setScissorWidth(width);
        GlobalRenderSystem.currentStateInfo.setScissorHeight(height);
        ci.cancel();
    }

    @Inject(method = "_enableCull", at = @At("HEAD"), cancellable = true)
    private static void enableCull(CallbackInfo ci) {
        GlobalRenderSystem.currentStateInfo.setCullEnabled(true);
        ci.cancel();
    }

    @Inject(method = "_disableCull", at = @At("HEAD"), cancellable = true)
    private static void disableCull(CallbackInfo ci) {
        GlobalRenderSystem.currentStateInfo.setCullEnabled(false);
        ci.cancel();
    }

    @Inject(method = "_depthFunc", at = @At("HEAD"), cancellable = true)
    private static void depthFunc(int func, CallbackInfo ci) {
        GlobalRenderSystem.currentStateInfo.setDepthCompareOp(GlConversions.glToVkDepthFunc(func));
        ci.cancel();
    }

    @Inject(method = "_enableBlend", at = @At("HEAD"), cancellable = true)
    private static void enableBlend(CallbackInfo ci) {
        GlobalRenderSystem.currentStateInfo.setBlendEnabled(true);
        ci.cancel();
    }

    @Inject(method = "_disableBlend", at = @At("HEAD"), cancellable = true)
    private static void disableBlend(CallbackInfo ci) {
        GlobalRenderSystem.currentStateInfo.setBlendEnabled(false);
        ci.cancel();
    }

    @Inject(method = "_blendEquation", at = @At("HEAD"), cancellable = true)
    private static void blendEquation(int mode, CallbackInfo ci) {
        GlobalRenderSystem.currentStateInfo.setBlendOp(GlConversions.glToVkBlendOp(mode));
        ci.cancel();
    }

    @Inject(method = "_blendFunc", at = @At("HEAD"), cancellable = true)
    private static void blendFunc(int srcFactor, int dstFactor, CallbackInfo ci) {
        int vkSrcFactor = GlConversions.glToVkBlendFunc(srcFactor);
        int vkDstFactor = GlConversions.glToVkBlendFunc(dstFactor);
        GlobalRenderSystem.currentStateInfo.setSrcColorBlendFactor(vkSrcFactor);
        GlobalRenderSystem.currentStateInfo.setDstColorBlendFactor(vkDstFactor);
        GlobalRenderSystem.currentStateInfo.setSrcAlphaBlendFactor(vkSrcFactor);
        GlobalRenderSystem.currentStateInfo.setDstAlphaBlendFactor(vkDstFactor);
        ci.cancel();
    }

    @Inject(method = "_blendFuncSeparate", at = @At("HEAD"), cancellable = true)
    private static void blendFunc(int srcFactorRGB, int dstFactorRGB, int srcFactorAlpha, int dstFactorAlpha, CallbackInfo ci) {
        GlobalRenderSystem.currentStateInfo.setSrcColorBlendFactor(GlConversions.glToVkBlendFunc(srcFactorRGB));
        GlobalRenderSystem.currentStateInfo.setDstColorBlendFactor(GlConversions.glToVkBlendFunc(dstFactorRGB));
        GlobalRenderSystem.currentStateInfo.setSrcAlphaBlendFactor(GlConversions.glToVkBlendFunc(srcFactorAlpha));
        GlobalRenderSystem.currentStateInfo.setDstAlphaBlendFactor(GlConversions.glToVkBlendFunc(dstFactorAlpha));
        ci.cancel();
    }

    @Inject(method = "_colorMask", at = @At("HEAD"), cancellable = true)
    private static void colorMask(boolean red, boolean green, boolean blue, boolean alpha, CallbackInfo ci) {
        int colorMask = 0;
        if (red) colorMask |= VK10.VK_COLOR_COMPONENT_R_BIT;
        if (green) colorMask |= VK10.VK_COLOR_COMPONENT_G_BIT;
        if (blue) colorMask |= VK10.VK_COLOR_COMPONENT_B_BIT;
        if (alpha) colorMask |= VK10.VK_COLOR_COMPONENT_A_BIT;
        GlobalRenderSystem.currentStateInfo.setColorMask(colorMask);
        ci.cancel();
    }

    @Inject(method = "_depthMask", at = @At("HEAD"), cancellable = true)
    private static void depthMask(boolean mask, CallbackInfo ci) {
        GlobalRenderSystem.currentStateInfo.setDepthMask(mask);
        ci.cancel();
    }

    @Inject(method = "_bindTexture", at = @At("HEAD"), cancellable = true)
    private static void bindTexture(int texture, CallbackInfo ci) {
        GlobalRenderSystem.boundTextureIds[GlobalRenderSystem.activeTexture] = texture;
        ci.cancel();
    }

    @Inject(method = "_getActiveTexture", at = @At("HEAD"), cancellable = true)
    private static void getActiveTexture(CallbackInfoReturnable<Integer> cir) {
        cir.setReturnValue(GlobalRenderSystem.activeTexture + GL13.GL_TEXTURE0);
    }

    @Inject(method = "_activeTexture", at = @At("HEAD"), cancellable = true)
    private static void activeTexture(int texture, CallbackInfo ci) {
        GlobalRenderSystem.activeTexture = texture - GL13.GL_TEXTURE0;
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
        Blaze4D.rosella.renderer.clearColor(new Color(red, green, blue, alpha));
    }
}
