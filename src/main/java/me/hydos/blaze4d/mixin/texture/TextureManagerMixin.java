package me.hydos.blaze4d.mixin.texture;

import me.hydos.blaze4d.Blaze4D;
import me.hydos.rosella.render.texture.SamplerCreateInfo;
import me.hydos.rosella.render.texture.TextureFilter;
import me.hydos.rosella.render.texture.UploadableImage;
import net.minecraft.client.texture.AbstractTexture;
import net.minecraft.client.texture.ResourceTexture;
import net.minecraft.client.texture.TextureManager;
import net.minecraft.util.Identifier;
import org.lwjgl.vulkan.VK10;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(TextureManager.class)
public class TextureManagerMixin {

    @Inject(method = "method_18167", at = @At("RETURN"))
    private void reloadTextures(CallbackInfo ci) {
        Blaze4D.rosella.reloadMaterials();
    }

    @Inject(method = "registerTexture", at = @At("HEAD"), cancellable = true)
    private void vkRegisterTexture(Identifier id, AbstractTexture texture, CallbackInfo ci) {
        try {
            if (!(texture instanceof ResourceTexture) && (((UploadableImage) texture).getPixels() != null)) {
                Blaze4D.rosella.getTextureManager().getOrLoadTexture(
                        (UploadableImage) texture,
                        Blaze4D.rosella,
                        switch (((UploadableImage) texture).getChannels()) {
                            case 4 -> VK10.VK_FORMAT_R32G32B32A32_SFLOAT;
                            case 3 -> VK10.VK_FORMAT_R32G32B32_SFLOAT;
                            case 2 -> VK10.VK_FORMAT_R32G32_SFLOAT;
                            case 1 -> VK10.VK_FORMAT_R32_SFLOAT;
                            default -> throw new IllegalStateException("Unexpected value: " + ((UploadableImage) texture).getChannels());
                        },
                        new SamplerCreateInfo(TextureFilter.NEAREST)
                );
            }
        } catch (ClassCastException e) {
            Blaze4D.LOGGER.warn("Class " + texture.getClass().getSimpleName() + " has no Rosella Texture Impl");
            // Ignore Classes we have not implemented for now
        } catch (Exception e) {
            Blaze4D.LOGGER.error("Something went wrong and fuck Mojang");
            e.printStackTrace();
        }
    }

    @Inject(method = "bindTexture", at = @At("HEAD"))
    private void vkBindTexture(Identifier id, CallbackInfo ci) {

    }
}
