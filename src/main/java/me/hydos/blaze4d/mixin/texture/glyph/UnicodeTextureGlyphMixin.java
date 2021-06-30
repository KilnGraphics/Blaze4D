package me.hydos.blaze4d.mixin.texture.glyph;

import me.hydos.blaze4d.api.texture.glyph.NativeImageBackedGlyph;
import net.minecraft.client.font.UnicodeTextureFont;
import net.minecraft.client.texture.NativeImage;
import org.jetbrains.annotations.Nullable;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;

@Mixin(UnicodeTextureFont.UnicodeTextureGlyph.class)
public class UnicodeTextureGlyphMixin implements NativeImageBackedGlyph {
    @Shadow
    @Final
    private NativeImage image;

    @Override
    @Nullable
    public NativeImage getBackedImage() {
        return image;
    }
}
