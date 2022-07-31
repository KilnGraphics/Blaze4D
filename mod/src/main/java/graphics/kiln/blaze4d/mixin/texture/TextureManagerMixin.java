package graphics.kiln.blaze4d.mixin.texture;

import net.minecraft.client.renderer.texture.TextureManager;
import org.spongepowered.asm.mixin.Mixin;

@Mixin(TextureManager.class)
public class TextureManagerMixin {

//    @Inject(method = "lambda$reload$4", at = @At("RETURN"), remap = false)
//    private void reloadTextures(CallbackInfo ci) {
//        Blaze4D.rosella.objectManager.submitMaterials();
//    }
}
