package me.hydos.blaze4d.mixin.texture;

import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import me.hydos.blaze4d.api.util.GlConversions;
import me.hydos.rosella.render.texture.*;
import me.hydos.rosella.scene.object.impl.SimpleObjectManager;
import net.minecraft.client.texture.NativeImage;
import org.jetbrains.annotations.NotNull;
import org.lwjgl.system.MemoryUtil;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.Unique;
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
    private long pointer;

    @Shadow
    @Final
    private long sizeBytes;

    @Shadow
    public abstract void close();

    @Unique
    private ImageFormat rosellaFormat;

    @Inject(method = "uploadInternal", at = @At("HEAD"), cancellable = true)
    private void uploadToRosella(int level, int offsetX, int offsetY, int unpackSkipPixels, int unpackSkipRows, int width, int height, boolean blur, boolean clamp, boolean mipmap, boolean close, CallbackInfo ci) {
        TextureManager textureManager = ((SimpleObjectManager) Blaze4D.rosella.objectManager).textureManager;
        textureManager.setTextureSampler(
                GlobalRenderSystem.boundTextureIds[GlobalRenderSystem.activeTexture],
                GlobalRenderSystem.activeTexture, // TODO: I think it's fine to assume texture no. here, but double check
                new SamplerCreateInfo(blur ? TextureFilter.LINEAR : TextureFilter.NEAREST, clamp ? WrapMode.CLAMP_TO_EDGE : WrapMode.REPEAT)
        );
        textureManager.drawToExistingTexture(
                Blaze4D.rosella.renderer,
                Blaze4D.rosella.common.memory,
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
            rosellaFormat = GlConversions.glToRosellaImageFormat(format.getPixelDataFormat()); // getPixelDataFormat returns the gl format
        }

        return rosellaFormat;
    }

    @Unique
    @Override
    public int getSize() {
        return (int) sizeBytes;
    }

    @Unique
    @Override
    public ByteBuffer getPixels() {
        return MemoryUtil.memByteBuffer(pointer, getSize());
    }
}
