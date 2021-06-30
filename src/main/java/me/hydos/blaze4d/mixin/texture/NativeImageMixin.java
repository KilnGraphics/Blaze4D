package me.hydos.blaze4d.mixin.texture;

import me.hydos.blaze4d.Blaze4D;
import me.hydos.rosella.render.texture.SamplerCreateInfo;
import me.hydos.rosella.render.texture.TextureFilter;
import me.hydos.rosella.render.texture.UploadableImage;
import net.minecraft.client.texture.NativeImage;
import org.lwjgl.system.MemoryUtil;
import org.lwjgl.vulkan.VK10;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.IntBuffer;

@Mixin(NativeImage.class)
public abstract class NativeImageMixin implements UploadableImage {

    @Shadow
    @Final
    private int width;

    @Shadow
    @Final
    private int height;

    @Shadow
    public abstract byte[] getBytes() throws IOException;

    @Shadow
    private long pointer;

    @Shadow
    public abstract NativeImage.Format getFormat();

    @Shadow
    public abstract void close();

    @Shadow public abstract int getPixelColor(int x, int y);

    private int channels = 4;
    private ByteBuffer pixels;

    @Inject(method = "<init>(Lnet/minecraft/client/texture/NativeImage$Format;IIZJ)V", at = @At("TAIL"))
    private void setExtraArgs(NativeImage.Format format, int width, int height, boolean useStb, long pointer, CallbackInfo ci) {
        this.channels = format.getChannelCount();
    }

    @Inject(method = "uploadInternal", at = @At("HEAD"), cancellable = true)
    private void uploadToRosella(int level, int offsetX, int offsetY, int unpackSkipPixels, int unpackSkipRows, int width, int height, boolean blur, boolean clamp, boolean mipmap, boolean close, CallbackInfo ci) {
        Blaze4D.rosella.getTextureManager().getOrLoadTexture(
                this,
                Blaze4D.rosella,
                switch (getFormat()) {
                    case ABGR -> VK10.VK_FORMAT_R32G32B32A32_SFLOAT;
                    case BGR -> VK10.VK_FORMAT_R32G32B32_SFLOAT;
                    case LUMINANCE_ALPHA -> VK10.VK_FORMAT_R32G32_SFLOAT;
                    case LUMINANCE -> VK10.VK_FORMAT_R32_SFLOAT;
                },
                new SamplerCreateInfo(blur ? TextureFilter.LINEAR : TextureFilter.NEAREST)
        );
        if (close) {
            this.close();
        }
        ci.cancel();
    }

    @Override
    public int getWidth() {
        return width;
    }

    @Override
    public int getHeight() {
        return height;
    }

    @Override
    public int getChannels() {
        return channels;
    }

    @Override
    public ByteBuffer getPixels() {
        if (pixels == null) {
            this.pixels = MemoryUtil.memAlloc(getImageSize());
            for (int y = 0; y < getHeight(); y++) {
                for (int x = 0; x < getWidth(); x++) {
                    int pixelColor = getPixelColor(x, y);
                    this.pixels.putFloat(NativeImage.getRed(pixelColor) / 255F);
                    this.pixels.putFloat(NativeImage.getGreen(pixelColor) / 255F);
                    this.pixels.putFloat(NativeImage.getBlue(pixelColor) / 255F);
                    this.pixels.putFloat(NativeImage.getAlpha(pixelColor) / 255F);
                }
            }
        }
        if (pixels.capacity() != getImageSize()) {
            throw new IllegalStateException("Image has wrong size! Expected: " + getImageSize() + " but got " + pixels.capacity());
        }

        return pixels;
    }

    @Override
    public int getImageSize() {
        return getWidth() * getHeight() * getChannels() * Float.BYTES;
    }
}
