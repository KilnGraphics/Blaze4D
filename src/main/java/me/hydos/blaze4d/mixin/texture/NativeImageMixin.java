package me.hydos.blaze4d.mixin.texture;

import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import me.hydos.rosella.render.texture.*;
import me.hydos.rosella.scene.object.impl.SimpleObjectManager;
import net.minecraft.client.texture.NativeImage;
import org.lwjgl.system.MemoryUtil;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.nio.ByteBuffer;

@Mixin(NativeImage.class)
public abstract class NativeImageMixin implements UploadableImage {

    @Shadow
    @Final
    private int width;

    @Shadow
    @Final
    private int height;

    @Shadow
    public abstract void close();

    @Shadow public abstract int getPixelColor(int x, int y);

    @Shadow @Final private NativeImage.Format format;

    @Inject(method = "uploadInternal", at = @At("HEAD"), cancellable = true)
    private void uploadToRosella(int level, int offsetX, int offsetY, int unpackSkipPixels, int unpackSkipRows, int width, int height, boolean blur, boolean clamp, boolean mipmap, boolean close, CallbackInfo ci) {
        TextureManager textureManager = ((SimpleObjectManager) Blaze4D.rosella.objectManager).textureManager;
        textureManager.applySamplerInfoToTexture(
                GlobalRenderSystem.boundTextureIds[GlobalRenderSystem.activeTexture],
                new SamplerCreateInfo(blur ? TextureFilter.LINEAR : TextureFilter.NEAREST)
        );
        textureManager.drawToExistingTexture(
                Blaze4D.rosella.renderer,
                Blaze4D.rosella.memory,
                GlobalRenderSystem.boundTextureIds[GlobalRenderSystem.activeTexture],
                this,
                new ImageRegion(width, height, unpackSkipPixels, unpackSkipRows),
                new ImageRegion(width, height, offsetX, offsetY)
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
        return format.getChannelCount();
    }

    @Override
    public ByteBuffer getPixels(ImageRegion region) {
        int imageSize = region.getWidth() * region.getHeight() * getBytesPerPixel();
        ByteBuffer pixels = MemoryUtil.memAlloc(imageSize);
        for (int y = region.getYOffset(); y < region.getHeight() + region.getYOffset(); y++) {
            for (int x = region.getXOffset(); x < region.getWidth() + region.getXOffset(); x++) {
                int pixelColor = getPixelColor(x, y);
                pixels.putFloat(NativeImage.getRed(pixelColor) / 255F);
                pixels.putFloat(NativeImage.getGreen(pixelColor) / 255F);
                pixels.putFloat(NativeImage.getBlue(pixelColor) / 255F);
                pixels.putFloat(NativeImage.getAlpha(pixelColor) / 255F);
            }
        }
        if (pixels.capacity() != imageSize) {
            throw new IllegalStateException("Image has wrong size! Expected: " + imageSize + " but got " + pixels.capacity());
        }

        return pixels;
    }

    @Override
    public int getBytesPerPixel() {
        return getChannels() * Float.BYTES;
    }
}
