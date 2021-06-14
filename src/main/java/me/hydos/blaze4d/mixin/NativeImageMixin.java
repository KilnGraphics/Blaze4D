package me.hydos.blaze4d.mixin;

import me.hydos.blaze4d.api.texture.Blaze4DNativeImage;
import net.minecraft.client.texture.NativeImage;
import org.jetbrains.annotations.Nullable;
import org.lwjgl.stb.STBImage;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.system.MemoryUtil;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.IntBuffer;

@Mixin(NativeImage.class)
public class NativeImageMixin implements Blaze4DNativeImage {

    private ByteBuffer imageBytes;

    /**
     * @author Blaze4d
     */
    @Overwrite
    public static NativeImage read(@Nullable NativeImage.Format format, ByteBuffer byteBuffer) throws IOException {
        if (format != null && !format.isWriteable()) {
            throw new UnsupportedOperationException("Don't know how to read format " + format);
        } else if (MemoryUtil.memAddress(byteBuffer) == 0L) {
            throw new IllegalArgumentException("Invalid buffer");
        } else {
            MemoryStack stack = MemoryStack.stackPush();

            NativeImage image;
            try {
                IntBuffer intBuffer = stack.mallocInt(1);
                IntBuffer intBuffer2 = stack.mallocInt(1);
                IntBuffer intBuffer3 = stack.mallocInt(1);
                ByteBuffer imageBytes = STBImage.stbi_load_from_memory(byteBuffer, intBuffer, intBuffer2, intBuffer3, format == null ? 0 : format.getChannelCount());
                if (imageBytes == null) {
                    throw new IOException("Could not load image: " + STBImage.stbi_failure_reason());
                }

                image = new NativeImage(format == null ? NativeImage.Format.getFormat(intBuffer3.get(0)) : format, intBuffer.get(0), intBuffer2.get(0), true, MemoryUtil.memAddress(imageBytes));
                ((Blaze4DNativeImage) (Object) image).setImageBuf(imageBytes);
            } catch (Throwable e) {
                try {
                    stack.close();
                } catch (Throwable var8) {
                    e.addSuppressed(var8);
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
    public void setImageBuf(ByteBuffer imageBytes) {
        this.imageBytes = imageBytes;
        if (imageBytes != null) {
//            Blaze4D.rosella.getTextureManager().getOrLoadTexture(
//                    Global.INSTANCE.fromByteBuffer(this.imageBytes, new Identifier("minecraft", this.hashCode() + "")),
//                    Blaze4D.rosella,
//                    VK10.VK_FORMAT_R8G8B8A8_SINT
//            );
        }
    }
}
