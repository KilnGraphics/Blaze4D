package graphics.kiln.blaze4d.mixin.texture;

import com.mojang.blaze3d.platform.GlStateManager;
import com.mojang.blaze3d.systems.RenderSystem;
import net.minecraft.client.Minecraft;
import net.minecraft.client.renderer.LightTexture;
import net.minecraft.resources.ResourceLocation;
import org.lwjgl.opengl.GL13;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(LightTexture.class)
public class LightmapTextureManagerMixin {

    @Shadow @Final private ResourceLocation lightTextureLocation;

    @Shadow @Final private Minecraft minecraft;

    @Inject(method = "turnOnLightLayer", at = @At("HEAD"), cancellable = true)
    private void removeBindTexture(CallbackInfo ci) {
        RenderSystem.setShaderTexture(2, this.lightTextureLocation);
        int prevActiveTexture = GlStateManager._getActiveTexture();
        RenderSystem.activeTexture(GL13.GL_TEXTURE2);
        this.minecraft.getTextureManager().bindForSetup(this.lightTextureLocation);
        RenderSystem.activeTexture(prevActiveTexture);
        RenderSystem.setShaderColor(1.0F, 1.0F, 1.0F, 1.0F);
        ci.cancel();
    }
}
