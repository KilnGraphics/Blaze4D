package me.hydos.blaze4d.mixin.texture;

import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.Materials;
import me.hydos.blaze4d.api.material.Blaze4dMaterial;
import me.hydos.rosella.render.texture.UploadableImage;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.texture.MissingSprite;
import net.minecraft.client.texture.NativeImage;
import net.minecraft.client.texture.ResourceTexture;
import net.minecraft.resource.ResourceManager;
import net.minecraft.util.Identifier;
import org.jetbrains.annotations.NotNull;
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

@Mixin(ResourceTexture.class)
public abstract class ResourceTextureMixin implements UploadableImage {

    @Shadow @Final protected Identifier location;

    @Shadow protected abstract ResourceTexture.TextureData loadTextureData(ResourceManager resourceManager);

    private NativeImage image;

    @Inject(method = "<init>", at = @At("TAIL"))
    private void loadNativeImage(Identifier location, CallbackInfo ci) {
        if(image == null) {
            this.image = MissingSprite.getMissingSpriteTexture().getImage();
//            ResourceTexture.TextureData data = this.loadTextureData(MinecraftClient.getInstance().getTextureManager().resourceContainer);
//            try {
//                this.image = data.getImage();
//            } catch (IOException e) {
//                this.image = MissingSprite.getMissingSpriteTexture().getImage();
////                throw new RuntimeException("Failed to get image", e);
//            }
        }
    }

    /**
     * @author Blaze4D
     * @reason Texture's
     * <p>
     * Cancel the upload method
     */
    @Overwrite
    public final void upload(NativeImage nativeImage, boolean blur, boolean clamp) {
        this.image = nativeImage;
        Blaze4D.rosella.getTextureManager().getOrLoadTexture(
                this,
                Blaze4D.rosella,
                VK10.VK_FORMAT_R8G8B8A8_SINT
        );
    }

    @Override
    public int getWidth() {
        return image.getWidth();
    }

    @Override
    public int getHeight() {
        return image.getHeight();
    }

    @Override
    public int getChannels() {
        return ((UploadableImage) (Object) image).getChannels();
    }

    @Override
    public ByteBuffer getPixels() {
        ByteBuffer pixels = ((UploadableImage) (Object) image).getPixels();
        if(pixels == null) {
            pixels = ByteBuffer.allocate(getImageSize());
            for (int x = 0; x < getWidth(); x++) {
                for (int y = 0; y < getHeight(); y++) {
                    pixels.putInt(image.getPixelColor(x, y));
                }
            }
        }
        return pixels;
    }

    @Override
    public int getImageSize() {
        return ((UploadableImage) (Object) image).getImageSize();
    }
}
