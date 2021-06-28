package me.hydos.blaze4d.mixin.texture;

import me.hydos.rosella.render.texture.UploadableImage;
import net.minecraft.client.texture.MissingSprite;
import net.minecraft.client.texture.NativeImage;
import net.minecraft.client.texture.ResourceTexture;
import net.minecraft.util.Identifier;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.nio.ByteBuffer;

@Mixin(ResourceTexture.class)
public abstract class ResourceTextureMixin implements UploadableImage {

    private NativeImage image;

    @Inject(method = "<init>", at = @At("TAIL"))
    private void loadNativeImage(Identifier location, CallbackInfo ci) {
        this.image = MissingSprite.getMissingSpriteTexture().getImage();
    }

    @Inject(method = "upload", at = @At("HEAD"))
    private void setImage(NativeImage nativeImage, boolean blur, boolean clamp, CallbackInfo ci) {
//        this.image = nativeImage;
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
        return ((UploadableImage) (Object) image).getPixels();
    }

    @Override
    public int getImageSize() {
        return getWidth() * getHeight() * getChannels();
    }
}
