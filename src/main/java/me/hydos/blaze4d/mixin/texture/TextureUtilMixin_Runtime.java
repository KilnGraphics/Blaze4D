package me.hydos.blaze4d.mixin.texture;

import com.mojang.blaze3d.platform.NativeImage;
import com.mojang.blaze3d.platform.TextureUtil;
import graphics.kiln.blaze4d.util.TextureUtils;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(value = TextureUtil.class, remap = false)
public class TextureUtilMixin_Runtime {

    @Inject(method = "prepareImage(Lnet/minecraft/class_1011$class_1013;IIII)V", at = @At("HEAD"), cancellable = true)
    private static void createRosellaTexture(NativeImage.InternalGlFormat internalFormat, int id, int maxLevel, int width, int height, CallbackInfo ci) {
        TextureUtils.createRosellaTexture(internalFormat, id, maxLevel, width, height, ci);
    }
}
