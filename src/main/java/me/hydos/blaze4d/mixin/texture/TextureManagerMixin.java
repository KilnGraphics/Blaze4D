package me.hydos.blaze4d.mixin.texture;

import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.Materials;
import me.hydos.blaze4d.api.material.Blaze4dMaterial;
import me.hydos.rosella.render.texture.UploadableImage;
import net.minecraft.client.texture.AbstractTexture;
import net.minecraft.client.texture.ResourceTexture;
import net.minecraft.client.texture.TextureManager;
import net.minecraft.util.Identifier;
import org.lwjgl.vulkan.VK10;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.util.Map;

@Mixin(TextureManager.class)
public class TextureManagerMixin {

    @Shadow
    @Final
    private Map<Identifier, AbstractTexture> textures;

    @Inject(method = "method_18167", at = @At("RETURN"))
    private void reloadTextures(CallbackInfo ci) {
        Blaze4D.rosella.reloadMaterials();
    }

    @Inject(method = "registerTexture", at = @At("HEAD"), cancellable = true)
    private void vkRegisterTexture(Identifier id, AbstractTexture texture, CallbackInfo ci) {
        Blaze4D.window.queue(() -> {
            try {
                if(!(texture instanceof ResourceTexture) && (((UploadableImage) texture).getPixels() != null)) {
                    Blaze4D.rosella.getTextureManager().getOrLoadTexture(
                            (UploadableImage) texture,
                            Blaze4D.rosella,
                            VK10.VK_FORMAT_R8G8B8A8_SINT
                    );
                }
            } catch (ClassCastException e) {
                System.out.println("Class " + texture.getClass().getSimpleName() + " has no Rosella Texture Impl");
                // Ignore Classes we have not implemented for now
            } catch (Exception e) {
                System.out.println("Something went wrong and fuck Mojang");
                e.printStackTrace();
            }
        });
        ci.cancel();
    }

    @Inject(method = "bindTexture", at = @At("HEAD"))
    private void vkBindTexture(Identifier id, CallbackInfo ci) {

    }
}
