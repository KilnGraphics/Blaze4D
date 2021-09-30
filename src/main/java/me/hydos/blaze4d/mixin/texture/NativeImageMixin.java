package me.hydos.blaze4d.mixin.texture;

import com.mojang.blaze3d.platform.NativeImage;
import graphics.kiln.rosella.render.texture.*;
import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import me.hydos.blaze4d.api.util.ConversionUtils;
import org.jetbrains.annotations.NotNull;
import org.lwjgl.system.MemoryUtil;
import org.spongepowered.asm.mixin.*;
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
    @Final
    private NativeImage.Format format;

    @Shadow
    private long pixels;

    @Shadow
    @Final
    private long size;

    @Shadow
    public abstract void close();

    @Unique
    private ImageFormat rosellaFormat;

    @Inject(method = "_upload", at = @At("HEAD"), cancellable = true)
    private void uploadToRosella(int level, int offsetX, int offsetY, int unpackSkipPixels, int unpackSkipRows, int width, int height, boolean blur, boolean clamp, boolean mipmap, boolean close, CallbackInfo ci) {
        TextureManager textureManager = Blaze4D.rosella.common.textureManager;
        textureManager.setTextureSampler(
                GlobalRenderSystem.boundTextureIds[GlobalRenderSystem.activeTextureSlot],
                GlobalRenderSystem.getSamplerNameForSlot(GlobalRenderSystem.activeTextureSlot),
                new SamplerCreateInfo(blur ? TextureFilter.LINEAR : TextureFilter.NEAREST, clamp ? WrapMode.CLAMP_TO_EDGE : WrapMode.REPEAT)
        );
        textureManager.drawToExistingTexture(
                Blaze4D.rosella.renderer,
                GlobalRenderSystem.boundTextureIds[GlobalRenderSystem.activeTextureSlot],
                this,
                new ImageRegion(width, height, unpackSkipPixels, unpackSkipRows),
                new ImageRegion(width, height, offsetX, offsetY)
        );
        if (close) {
            this.close();
        }
        ci.cancel();
    }

    @Unique
    @Override
    public int getWidth() {
        return width;
    }

    @Unique
    @Override
    public int getHeight() {
        return height;
    }

    @Unique
    @NotNull
    @Override
    public ImageFormat getFormat() {
        if (rosellaFormat == null) {
            rosellaFormat = ConversionUtils.glToRosellaImageFormat(format.glFormat()); // getPixelDataFormat returns the gl format
        }

        return rosellaFormat;
    }

    @Unique
    @Override
    public int getSize() {
        return (int) size;
    }

    @Unique
    @Override
    public ByteBuffer getPixels() {
        return MemoryUtil.memByteBuffer(pixels, getSize());
    }

    /**
     * @author ramidzkh
     * @reason Don't die screenshotting
     */
    @Overwrite
    public void downloadTexture(int lod, boolean captureAlpha) {
    }
}
