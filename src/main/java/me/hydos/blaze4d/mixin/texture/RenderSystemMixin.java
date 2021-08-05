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
    private static void setTextureFromIdentifier(int slot, ResourceLocation identifier, CallbackInfo ci) {
        setTexture(slot, identifier, ci);
    }

    @Inject(method = "setShaderTexture(ILnet/minecraft/class_2960;)V", at = @At("HEAD"), require = 0, cancellable = true) // ugly hack to get around mixin not remapping properly
    private static void setTextureFromIdentifierInIntermediary(int slot, ResourceLocation identifier, CallbackInfo ci) {
        setTexture(slot, identifier, ci);
    }

    private static void setTexture(int slot, ResourceLocation identifier, CallbackInfo ci) {
        if (slot >= 0 && slot < GlobalRenderSystem.MAX_TEXTURES) {
            TextureManager textureManager = Minecraft.getInstance().getTextureManager();
            AbstractTexture abstractTexture = textureManager.getTexture(identifier);
            GlobalRenderSystem.boundTextureIds[slot] = abstractTexture.getId();
        }
        ci.cancel();
    }

    @Inject(method = "setShaderTexture(II)V", at = @At("HEAD"), cancellable = true)
    private static void setTextureFromId(int slot, int texId, CallbackInfo ci) {
        if (slot >= 0 && slot < GlobalRenderSystem.MAX_TEXTURES) {
            GlobalRenderSystem.boundTextureIds[slot] = texId;
        }
        ci.cancel();
    }

    @Inject(method = "getShaderTexture", at = @At("HEAD"), cancellable = true)
    private static void getTextureFromUs(int slot, CallbackInfoReturnable<Integer> cir) {
        cir.setReturnValue(slot >= 0 && slot < GlobalRenderSystem.MAX_TEXTURES ? GlobalRenderSystem.boundTextureIds[slot] : 0);
    }
}
