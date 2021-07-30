package me.hydos.blaze4d.mixin.integration;

import com.mojang.blaze3d.platform.GlStateManager;
import com.mojang.blaze3d.systems.RenderSystem;
import com.sun.jna.platform.win32.GL;
import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import me.hydos.blaze4d.api.util.ConversionUtils;
import me.hydos.rosella.scene.object.impl.SimpleObjectManager;
import me.hydos.rosella.util.Color;
import org.lwjgl.opengl.GL13;
import org.lwjgl.vulkan.VK10;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(value = GlStateManager.class, remap = false)
public class GlStateManagerMixin {

    @Inject(method = {
            "_texParameter(IIF)V",
            "_texParameter(III)V",
            "_texImage2D",
            "_clear",
            "_glBindFramebuffer",
            "_polygonOffset",
            "_polygonMode",
            "_enablePolygonOffset",
            "_disablePolygonOffset",
            "_viewport"
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
        GlobalRenderSystem.currentStateInfo.setColorLogicOp(ConversionUtils.glToVkLogicOp(op));
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
        GlobalRenderSystem.currentStateInfo.setDepthCompareOp(ConversionUtils.glToVkDepthFunc(func));
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
        GlobalRenderSystem.currentStateInfo.setBlendOp(ConversionUtils.glToVkBlendOp(mode));
        ci.cancel();
    }

    @Inject(method = "_blendFunc", at = @At("HEAD"), cancellable = true)
    private static void blendFunc(int srcFactor, int dstFactor, CallbackInfo ci) {
        int vkSrcFactor = ConversionUtils.glToVkBlendFunc(srcFactor);
        int vkDstFactor = ConversionUtils.glToVkBlendFunc(dstFactor);
        GlobalRenderSystem.currentStateInfo.setSrcColorBlendFactor(vkSrcFactor);
        GlobalRenderSystem.currentStateInfo.setDstColorBlendFactor(vkDstFactor);
        GlobalRenderSystem.currentStateInfo.setSrcAlphaBlendFactor(vkSrcFactor);
        GlobalRenderSystem.currentStateInfo.setDstAlphaBlendFactor(vkDstFactor);
        ci.cancel();
    }

    @Inject(method = "_blendFuncSeparate", at = @At("HEAD"), cancellable = true)
    private static void blendFunc(int srcFactorRGB, int dstFactorRGB, int srcFactorAlpha, int dstFactorAlpha, CallbackInfo ci) {
        GlobalRenderSystem.currentStateInfo.setSrcColorBlendFactor(ConversionUtils.glToVkBlendFunc(srcFactorRGB));
        GlobalRenderSystem.currentStateInfo.setDstColorBlendFactor(ConversionUtils.glToVkBlendFunc(dstFactorRGB));
        GlobalRenderSystem.currentStateInfo.setSrcAlphaBlendFactor(ConversionUtils.glToVkBlendFunc(srcFactorAlpha));
        GlobalRenderSystem.currentStateInfo.setDstAlphaBlendFactor(ConversionUtils.glToVkBlendFunc(dstFactorAlpha));
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
    private static void bindTexture(int texId, CallbackInfo ci) {
        GlobalRenderSystem.setTextureIdInSlot(GlobalRenderSystem.getActiveTextureSlot(), texId);
        ci.cancel();
    }

    @Inject(method = "_getActiveTexture", at = @At("HEAD"), cancellable = true)
    private static void getActiveTexture(CallbackInfoReturnable<Integer> cir) {
        cir.setReturnValue(GlobalRenderSystem.getActiveTextureSlot() + GL13.GL_TEXTURE0);
    }

    @Inject(method = "_activeTexture", at = @At("HEAD"), cancellable = true)
    private static void activeTexture(int texSlot, CallbackInfo ci) {
        GlobalRenderSystem.setActiveTextureSlot(texSlot - GL13.GL_TEXTURE0);
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

    @Inject(method = "_clearStencil", at = @At("HEAD"), cancellable = true)
    private static void clearStencil(int stencil, CallbackInfo ci) {
        Blaze4D.rosella.renderer.lazilyClearStencil(stencil); // TODO: should this value be converted ogl to vk?
        ci.cancel();
    }

    @Inject(method = "_clearDepth", at = @At("HEAD"), cancellable = true)
    private static void clearDepth(double depth, CallbackInfo ci) {
        Blaze4D.rosella.renderer.lazilyClearDepth((float) depth);
        ci.cancel();
    }

    @Inject(method = "_getString", at = @At("HEAD"), cancellable = true)
    private static void getString(int glStringId, CallbackInfoReturnable<String> ci) {
        ci.setReturnValue(
                Blaze4D.rosella == null ? "Device not initialized" :
                    switch (glStringId) {
                        case GL.GL_VENDOR -> tryParseVendorId(Blaze4D.rosella.common.device.properties.vendorId);
                        case GL.GL_EXTENSIONS -> Blaze4D.rosella.common.device.combinedExtensionsString;
                        case GL.GL_RENDERER -> Blaze4D.rosella.common.device.properties.deviceName;
                        case GL.GL_VERSION -> "Vulkan API " + Blaze4D.rosella.common.device.properties.apiVersion;
                        default -> throw new IllegalStateException("Unexpected value: " + glStringId);
                    }
        );
    }

    @Unique
    private static String tryParseVendorId(int vendorId) {
        return switch (vendorId) {
            case 0x10DE -> "NVIDIA Corporation";
            case 0x1002 -> "AMD";
            case 0x8086 -> "INTEL";
            case 0x13B5 -> "ARM";
            case 0x1010 -> "ImgTec";
            case 0x5143 -> "Qualcomm";
            default -> "Vendor unknown. Vendor ID: " + vendorId;
        };
    }

    /**
     * @author Blaze4D
     * @reason Clear Color Integration
     * <p>
     * Minecraft may be regarded as having bad code, but sometimes its ok.
     * TODO: use vkCmdClearAttachments after implementing render graph
     */
    @Overwrite
    public static void _clearColor(float red, float green, float blue, float alpha) {
        RenderSystem.assertThread(RenderSystem::isOnRenderThreadOrInit);
        Blaze4D.rosella.renderer.lazilyClearColor(new Color(red, green, blue, alpha));
    }

    @Overwrite
    public static int _glGenVertexArrays() {
        return 0;
    }

    @Overwrite
    public static void _glBindVertexArray(int i) {
    }

    @Overwrite
    public static void _disableVertexAttribArray(int index) {
    }
}
