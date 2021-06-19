package me.hydos.blaze4d.mixin.texture;

import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.Materials;
import me.hydos.blaze4d.api.material.Blaze4dMaterial;
import me.hydos.rosella.render.texture.UploadableImage;
import net.minecraft.client.texture.NativeImage;
import net.minecraft.client.texture.ResourceTexture;
import net.minecraft.util.Identifier;
import org.jetbrains.annotations.NotNull;
import org.lwjgl.vulkan.VK10;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;

import java.nio.ByteBuffer;

@Mixin(ResourceTexture.class)
public class ResourceTextureMixin implements UploadableImage {

    @Shadow @Final protected Identifier location;
    private NativeImage image;

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

        Blaze4D.rosella.registerMaterial(
                new me.hydos.rosella.render.resource.Identifier(location.getNamespace(), location.getPath()),
                new Blaze4dMaterial(Materials.SOLID_COLOR_TRIANGLES, this)
        );
    }

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

    @NotNull
    @Override
    public ByteBuffer getPixels() {
        return ((UploadableImage) (Object) image).getPixels();
    }

    @Override
    public int getImageSize() {
        return ((UploadableImage) (Object) image).getImageSize();
    }
}
