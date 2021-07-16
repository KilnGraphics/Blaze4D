package me.hydos.blaze4d.mixin.texture;

import com.mojang.blaze3d.platform.NativeImage;
import com.mojang.blaze3d.platform.TextureUtil;
import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import me.hydos.blaze4d.api.util.ConversionUtils;
import me.hydos.rosella.scene.object.impl.SimpleObjectManager;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(TextureUtil.class)
public class TextureUtilMixin {

    @Inject(method = "prepareImage(Lcom/mojang/blaze3d/platform/NativeImage$InternalGlFormat;IIII)V", at = @At("HEAD"), remap = false, cancellable = true)
    private static void createRosellaTexture(NativeImage.InternalGlFormat internalFormat, int id, int maxLevel, int width, int height, CallbackInfo ci) {
        GlobalRenderSystem.boundTextureIds[GlobalRenderSystem.activeTexture] = id;
        ((SimpleObjectManager) Blaze4D.rosella.objectManager).textureManager.createTexture(
                Blaze4D.rosella.renderer,
                Blaze4D.rosella.common.memory,
                id,
                width,
                height,
                ConversionUtils.glToVkDefaultImageFormat(internalFormat.glFormat())
        );
        ci.cancel();
    }
}
