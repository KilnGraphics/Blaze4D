package me.hydos.blaze4d.mixin.texture;

import me.hydos.blaze4d.Blaze4D;
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

    @Shadow private long pointer;

    @Shadow public abstract NativeImage.Format getFormat();

    private int channels = 4;
    private ByteBuffer pixels;

    @Inject(method = "<init>(Lnet/minecraft/client/texture/NativeImage$Format;IIZJ)V", at = @At("TAIL"))
    private void setExtraArgs(NativeImage.Format format, int width, int height, boolean useStb, long pointer, CallbackInfo ci) throws IOException {
        this.pixels = ByteBuffer.wrap(getBytes());
        this.channels = format.getChannelCount();
    }

    @Inject(method = "uploadInternal", at = @At("HEAD"), cancellable = true)
    private void uploadToRosella(int level, int offsetX, int offsetY, int unpackSkipPixels, int unpackSkipRows, int width, int height, boolean blur, boolean clamp, boolean mipmap, boolean close, CallbackInfo ci) {
//        Blaze4D.rosella.getTextureManager().getOrLoadTexture(
//                this,
//                Blaze4D.rosella,
//                switch (getFormat()) {
//                    case ABGR -> VK10.VK_FORMAT_R32G32B32A32_SFLOAT;
//                    case BGR -> VK10.VK_FORMAT_R32G32B32_SFLOAT;
//                    case LUMINANCE_ALPHA -> VK10.VK_FORMAT_R32G32_SFLOAT;
//                    case LUMINANCE -> VK10.VK_FORMAT_R32_SFLOAT;
//                },
//                blur ? VK10.VK_FILTER_LINEAR : VK10.VK_FILTER_NEAREST
//        );
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
            this.pixels = MemoryUtil.memAlloc(getImageSize() * Float.BYTES);
            ByteBuffer originalIntBytes = MemoryUtil.memByteBuffer(pointer, getImageSize() / 4);
            for (int i = 0; i < originalIntBytes.limit(); i++) {
                this.pixels.putFloat(Byte.toUnsignedInt(originalIntBytes.get(i)) / 255F);
            }
        }
        return pixels;
    }

    @Override
    public int getImageSize() {
        return getWidth() * getHeight() * getChannels() * Float.BYTES;
    }
}
