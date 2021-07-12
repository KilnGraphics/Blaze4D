package me.hydos.blaze4d.mixin.texture;

import com.mojang.blaze3d.platform.GlStateManager;
import com.mojang.blaze3d.systems.RenderSystem;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.render.LightmapTextureManager;
import net.minecraft.util.Identifier;
import org.lwjgl.opengl.GL13;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(LightmapTextureManager.class)
public class LightmapTextureManagerMixin {

    @Shadow @Final private Identifier textureIdentifier;

    @Shadow @Final private MinecraftClient client;

    @Inject(method = "enable", at = @At("HEAD"), cancellable = true)
    private void removeBindTexture(CallbackInfo ci) {
        RenderSystem.setShaderTexture(2, this.textureIdentifier);
        int prevActiveTexture = GlStateManager._getActiveTexture();
        RenderSystem.activeTexture(GL13.GL_TEXTURE2);
        this.client.getTextureManager().bindTexture(this.textureIdentifier);
        RenderSystem.activeTexture(prevActiveTexture);
        RenderSystem.setShaderColor(1.0F, 1.0F, 1.0F, 1.0F);
        ci.cancel();
    }
}
