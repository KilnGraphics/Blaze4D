package graphics.kiln.blaze4d.util;

import com.mojang.blaze3d.platform.NativeImage;
import graphics.kiln.blaze4d.Blaze4D;
import graphics.kiln.blaze4d.impl.GlobalRenderSystem;
import me.hydos.blaze4d.api.util.ConversionUtils;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

// TODO: Remove once we can use Mixin 0.8.3 @Desc
@Deprecated
public class TextureUtils {

    public static void createRosellaTexture(NativeImage.InternalGlFormat internalFormat, int id, int maxLevel, int width, int height, CallbackInfo ci) {
        GlobalRenderSystem.boundTextureIds[GlobalRenderSystem.activeTextureSlot] = id;
        Blaze4D.rosella.common.textureManager.createTexture(
                Blaze4D.rosella.renderer,
                id,
                width,
                height,
                ConversionUtils.glToVkDefaultImageFormat(internalFormat.glFormat())
        );
        ci.cancel();
    }
}
