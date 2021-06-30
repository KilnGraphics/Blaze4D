package me.hydos.blaze4d.mixin.texture.glyph;

import me.hydos.blaze4d.api.texture.glyph.NativeImageBackedGlyph;
import net.minecraft.client.font.TrueTypeFont;
import net.minecraft.client.texture.NativeImage;
import org.jetbrains.annotations.Nullable;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.LocalCapture;

@Mixin(TrueTypeFont.TtfGlyph.class)
public class TtfGlyphMixin implements NativeImageBackedGlyph {
    @Unique private NativeImage image;

    @Inject(method = "upload", at = @At(value = "INVOKE", shift = At.Shift.AFTER, target = "Lnet/minecraft/client/texture/NativeImage;makeGlyphBitmapSubpixel(Lorg/lwjgl/stb/STBTTFontinfo;IIIFFFFII)V"), locals = LocalCapture.CAPTURE_FAILHARD)
    private void captureNativeImage(int x, int y, CallbackInfo ci, NativeImage image) {
        this.image = image;
    }

    @Override
    @Nullable
    public NativeImage getBackedImage() {
        return image;
    }
}
