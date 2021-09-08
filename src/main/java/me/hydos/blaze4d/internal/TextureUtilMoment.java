package me.hydos.blaze4d.internal;

import com.mojang.blaze3d.platform.NativeImage;
import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import me.hydos.blaze4d.api.util.ConversionUtils;
import me.hydos.rosella.scene.object.impl.SimpleObjectManager;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

// TODO: Remove once we can use Mixin 0.8.3 @Desc
public class TextureUtilMoment {

    public static void createRosellaTexture(NativeImage.InternalGlFormat internalFormat, int id, int maxLevel, int width, int height, CallbackInfo ci) {
        GlobalRenderSystem.boundTextureIds[GlobalRenderSystem.activeTextureSlot] = id;
        ((SimpleObjectManager) Blaze4D.rosella.objectManager).textureManager.createTexture(
                Blaze4D.rosella.renderer,
                id,
                width,
                height,
                ConversionUtils.glToVkDefaultImageFormat(internalFormat.glFormat())
        );
        ci.cancel();
    }
}
