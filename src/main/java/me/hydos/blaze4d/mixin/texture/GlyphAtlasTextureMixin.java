package me.hydos.blaze4d.mixin.texture;

import me.hydos.rosella.render.texture.UploadableImage;
import net.minecraft.client.font.GlyphAtlasTexture;
import net.minecraft.client.texture.AbstractTexture;
import net.minecraft.util.Identifier;
import org.jetbrains.annotations.NotNull;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.nio.ByteBuffer;

@Mixin(GlyphAtlasTexture.class)
public class GlyphAtlasTextureMixin {// implements UploadableImage {
    @Shadow
    @Final
    private Identifier id;

//    @Override
//    public int getWidth() {
//        return ((UploadableImage) (Object) image).getWidth();
//    }
//
//    @Override
//    public int getHeight() {
//        return ((UploadableImage) (Object) image).getHeight();
//    }
//
//    @Override
//    public int getChannels() {
//        return ((UploadableImage) (Object) image).getChannels();
//    }
//
//    @NotNull
//    @Override
//    public ByteBuffer getPixels() {
//        return ((UploadableImage) (Object) image).getPixels();
//    }
//
//    @Override
//    public int getImageSize() {
//        return ((UploadableImage) (Object) image).getImageSize();
//    }

    @Unique
    private AbstractTexture image;

    @Inject(method = "<init>", at = @At("RETURN"))
    private void captureNativeImage(CallbackInfo info) {
//        image = MinecraftClient.getInstance().getTextureManager().getTexture(id);
    }
}
