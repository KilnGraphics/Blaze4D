package me.hydos.blaze4d.mixin.texture;

import com.mojang.blaze3d.platform.TextureUtil;
import net.minecraft.client.texture.NativeImage;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(TextureUtil.class)
public class TextureUtilMixin {

    @Inject(method = "prepareImage(Lnet/minecraft/client/texture/NativeImage$GLFormat;IIII)V", at = @At("HEAD"), cancellable = true)
    private static void loadTextureIntoRosella(NativeImage.GLFormat internalFormat, int id, int maxLevel, int width, int height, CallbackInfo ci) {
        // TODO: impl
        ci.cancel();
    }
}
