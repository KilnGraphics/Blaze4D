package me.hydos.blaze4d.mixin.texture;

import com.mojang.blaze3d.platform.NativeImage;
import com.mojang.blaze3d.platform.TextureUtil;
import me.hydos.blaze4d.internal.TextureUtilMoment;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(value = TextureUtil.class, remap = false)
public class TextureUtilMixin_Development {

    @Inject(method = "prepareImage(Lcom/mojang/blaze3d/platform/NativeImage$InternalGlFormat;IIII)V", at = @At("HEAD"), cancellable = true)
    private static void createRosellaTexture(NativeImage.InternalGlFormat internalFormat, int id, int maxLevel, int width, int height, CallbackInfo ci) {
        TextureUtilMoment.createRosellaTexture(internalFormat, id, maxLevel, width, height, ci);
    }
}
