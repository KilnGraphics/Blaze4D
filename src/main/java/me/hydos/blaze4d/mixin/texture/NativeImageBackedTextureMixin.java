package me.hydos.blaze4d.mixin.texture;

import me.hydos.rosella.render.texture.UploadableImage;
import net.minecraft.client.texture.NativeImage;
import net.minecraft.client.texture.NativeImageBackedTexture;
import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;

import java.nio.ByteBuffer;

@Mixin(NativeImageBackedTexture.class)
public class NativeImageBackedTextureMixin implements UploadableImage {

    @Shadow
    @Nullable
    private NativeImage image;

    @Override
    public int getWidth() {
        return ((UploadableImage) (Object) image).getWidth();
    }

    @Override
    public int getHeight() {
        return ((UploadableImage) (Object) image).getHeight();
    }

    @Override
    public int getChannels() {
        return ((UploadableImage) (Object) image).getChannels();
    }

    @Override
    public ByteBuffer getPixels() {
        return ((UploadableImage) (Object) image).getPixels();
    }

    @Override
    public int getImageSize() {
        return ((UploadableImage) (Object) image).getImageSize();
    }
}
