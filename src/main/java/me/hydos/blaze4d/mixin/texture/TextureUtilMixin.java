package me.hydos.blaze4d.mixin.texture;

import com.mojang.blaze3d.platform.TextureUtil;
import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import me.hydos.rosella.render.texture.SamplerCreateInfo;
import me.hydos.rosella.render.texture.TextureFilter;
import net.minecraft.client.texture.NativeImage;
import org.lwjgl.vulkan.VK10;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(TextureUtil.class)
public class TextureUtilMixin {

    @Inject(method = "prepareImage(Lnet/minecraft/client/texture/NativeImage$GLFormat;IIII)V", at = @At("HEAD"), cancellable = true)
    private static void createRosellaTexture(NativeImage.GLFormat internalFormat, int id, int maxLevel, int width, int height, CallbackInfo ci) {
        GlobalRenderSystem.boundTextureId = id;
        Blaze4D.rosella.getTextureManager().createTexture(
                Blaze4D.rosella,
                id,
                width,
                height,
                switch (internalFormat) {
                    case ABGR -> VK10.VK_FORMAT_R32G32B32A32_SFLOAT;
                    case BGR -> VK10.VK_FORMAT_R32G32B32_SFLOAT;
                    case RG -> VK10.VK_FORMAT_R32G32_SFLOAT;
                    case RED -> VK10.VK_FORMAT_R32_SFLOAT;
                },
                new SamplerCreateInfo(TextureFilter.NEAREST) // TODO: hmm...
        );
        GlobalRenderSystem.boundTextureId = id;
        ci.cancel();
    }
}
