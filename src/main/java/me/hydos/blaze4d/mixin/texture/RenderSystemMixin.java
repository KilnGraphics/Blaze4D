package me.hydos.blaze4d.mixin.texture;

import com.mojang.blaze3d.systems.RenderSystem;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import net.minecraft.client.Minecraft;
import net.minecraft.client.renderer.texture.AbstractTexture;
import net.minecraft.client.renderer.texture.TextureManager;
import net.minecraft.resources.ResourceLocation;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(value = RenderSystem.class, remap = false)
public class RenderSystemMixin {

    @Inject(method = "setShaderTexture(ILnet/minecraft/resources/ResourceLocation;)V", at = @At("HEAD"), require = 0, cancellable = true)
    private static void setTextureFromIdentifier(int i, ResourceLocation identifier, CallbackInfo ci) {
        setTexture(i, identifier, ci);
    }

    @Inject(method = "setShaderTexture(ILnet/minecraft/class_2960;)V", at = @At("HEAD"), require = 0, cancellable = true) // ugly hack to get around mixin not remapping properly
    private static void setTextureFromIdentifierInIntermediary(int i, ResourceLocation identifier, CallbackInfo ci) {
        setTexture(i, identifier, ci);
    }

    private static void setTexture(int i, ResourceLocation identifier, CallbackInfo ci) {
        if (i >= 0 && i < GlobalRenderSystem.boundTextureIds.length) {
            TextureManager textureManager = Minecraft.getInstance().getTextureManager();
            AbstractTexture abstractTexture = textureManager.getTexture(identifier);
            GlobalRenderSystem.boundTextureIds[i] = abstractTexture.getId();
        }
        ci.cancel();
    }

    @Inject(method = "setShaderTexture(II)V", at = @At("HEAD"), cancellable = true)
    private static void setTextureFromId(int i, int j, CallbackInfo ci) {
        if (i >= 0 && i < GlobalRenderSystem.boundTextureIds.length) {
            GlobalRenderSystem.boundTextureIds[i] = j;
        }
        ci.cancel();
    }

    @Inject(method = "getShaderTexture", at = @At("HEAD"), cancellable = true)
    private static void getTextureFromUs(int i, CallbackInfoReturnable<Integer> cir) {
        cir.setReturnValue(i >= 0 && i < GlobalRenderSystem.boundTextureIds.length ? GlobalRenderSystem.boundTextureIds[i] : 0);
    }
}
