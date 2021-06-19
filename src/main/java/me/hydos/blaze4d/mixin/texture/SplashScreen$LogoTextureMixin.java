package me.hydos.blaze4d.mixin.texture;

import net.minecraft.client.gui.screen.SplashScreen;
import net.minecraft.client.texture.ResourceTexture;
import net.minecraft.resource.ResourceManager;
import net.minecraft.util.Identifier;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.io.IOException;

@Mixin(SplashScreen.LogoTexture.class)
public abstract class SplashScreen$LogoTextureMixin extends ResourceTexture {

    public SplashScreen$LogoTextureMixin(Identifier location) {
        super(location);
    }

    @Inject(method = "loadTextureData", at = @At("RETURN"))
    private void uploadTexture(ResourceManager resourceManager, CallbackInfoReturnable<TextureData> cir) throws IOException {
        this.upload(cir.getReturnValue().getImage(), false, false);
    }
}
