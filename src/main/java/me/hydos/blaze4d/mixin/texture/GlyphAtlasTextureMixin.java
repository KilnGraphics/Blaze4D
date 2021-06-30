package me.hydos.blaze4d.mixin.texture;

import me.hydos.blaze4d.api.texture.glyph.NativeImageBackedGlyph;
import me.hydos.rosella.render.texture.UploadableImage;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.font.GlyphAtlasTexture;
import net.minecraft.client.font.GlyphRenderer;
import net.minecraft.client.font.RenderableGlyph;
import net.minecraft.client.texture.AbstractTexture;
import net.minecraft.client.texture.MissingSprite;
import net.minecraft.client.texture.NativeImage;
import net.minecraft.util.Identifier;
import org.jetbrains.annotations.NotNull;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.nio.ByteBuffer;

@Mixin(GlyphAtlasTexture.class)
public class GlyphAtlasTextureMixin implements UploadableImage {
    @Unique
    private NativeImage currentImage;

    @Inject(method = "<init>", at = @At("TAIL"))
    private void setDefaultTexture(Identifier id, boolean hasColor, CallbackInfo ci) {
        currentImage = MissingSprite.getMissingSpriteTexture().getImage();
    }

    @Override
    public int getWidth() {
        return ((UploadableImage) (Object) currentImage).getWidth();
    }

    @Override
    public int getHeight() {
        return ((UploadableImage) (Object) currentImage).getHeight();
    }

    @Override
    public int getChannels() {
        return ((UploadableImage) (Object) currentImage).getChannels();
    }

    @NotNull
    @Override
    public ByteBuffer getPixels() {
        return ((UploadableImage) (Object) currentImage).getPixels();
    }

    @Override
    public int getImageSize() {
        return ((UploadableImage) (Object) currentImage).getImageSize();
    }

    @Inject(method = "getGlyphRenderer", at = @At(value = "HEAD"))
    private void extractImageFromGlyph(RenderableGlyph glyph, CallbackInfoReturnable<GlyphRenderer> cir) {
        if (glyph instanceof NativeImageBackedGlyph backedGlyph) {
            currentImage = backedGlyph.getBackedImage();
        } else {
            throw new IllegalArgumentException("glyph not instance of NativeImageBackedGlyph");
        }
    }
}
