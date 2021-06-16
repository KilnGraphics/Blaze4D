package me.hydos.blaze4d.mixin.texture;

import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.texture.Blaze4dNativeImage;
import me.hydos.rosella.render.texture.UploadableImage;
import net.minecraft.client.texture.NativeImage;
import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;
import org.lwjgl.stb.STBImage;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.system.MemoryUtil;
import org.lwjgl.vulkan.VK10;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.IntBuffer;

@Mixin(NativeImage.class)
public class NativeImageMixin implements UploadableImage, Blaze4dNativeImage {

    @Shadow
    @Final
    private int width;

    @Shadow
    @Final
    private int height;

    private int channels;
    private ByteBuffer pixels;

    /**
     * @author Blaze4d
     */
    @Overwrite
    public static NativeImage read(@Nullable NativeImage.Format format, ByteBuffer pixels) throws IOException {
        if (format != null && !format.isWriteable()) {
            throw new UnsupportedOperationException("Don't know how to read format " + format);
        } else if (MemoryUtil.memAddress(pixels) == 0L) {
            throw new IllegalArgumentException("Invalid buffer");
        } else {
            MemoryStack stack = MemoryStack.stackPush();

            NativeImage image;
            try {
                IntBuffer pWidth = stack.mallocInt(1);
                IntBuffer pHeight = stack.mallocInt(1);
                IntBuffer pChannels = stack.mallocInt(1);
                int desiredChannels = format == null ? 0 : format.getChannelCount();
                ByteBuffer imageBytes = STBImage.stbi_load_from_memory(pixels, pWidth, pHeight, pChannels, desiredChannels);
                if (imageBytes == null) {
                    throw new IOException("Could not load image: " + STBImage.stbi_failure_reason());
                }

                int channels = pChannels.get(0);
                image = new NativeImage(format == null ? NativeImage.Format.getFormat(channels) : format, pWidth.get(0), pHeight.get(0), true, MemoryUtil.memAddress(imageBytes));
                Blaze4dNativeImage uploadableImage = (Blaze4dNativeImage) (Object) image;
                uploadableImage.setChannels(channels);
                uploadableImage.setPixels(pixels);
            } catch (Throwable e) {
                try {
                    stack.close();
                } catch (Throwable t) {
                    e.addSuppressed(t);
                }
                throw e;
            }
            stack.close();
            return image;
        }
    }

    @Inject(method = "uploadInternal", at = @At("HEAD"), cancellable = true)
    private void uploadToRosella(int level, int offsetX, int offsetY, int unpackSkipPixels, int unpackSkipRows, int width, int height, boolean blur, boolean clamp, boolean mipmap, boolean close, CallbackInfo ci) {
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
    public @NotNull ByteBuffer getPixels() {
        return pixels;
    }

    @Override
    public int getImageSize() {
        return width * height * 4;
    }

    @Override
    public void setChannels(int channels) {
        this.channels = channels;
    }

    @Override
    public void setPixels(ByteBuffer pixels) {
        this.pixels = pixels;
        Blaze4D.window.queue(() -> Blaze4D.rosella.getTextureManager().getOrLoadTexture(
                this,
                Blaze4D.rosella,
                VK10.VK_FORMAT_R8G8B8A8_SINT
        ));
    }
}
