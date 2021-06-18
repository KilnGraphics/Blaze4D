package me.hydos.blaze4d.mixin.texture;

import me.hydos.blaze4d.Blaze4D;
import net.minecraft.client.texture.TextureManager;
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
}
