package me.hydos.blaze4d.mixin.texture.glyph;

import me.hydos.blaze4d.api.texture.glyph.NativeImageBackedGlyph;
import net.minecraft.client.font.WhiteRectangleGlyph;
import net.minecraft.client.texture.NativeImage;
import org.jetbrains.annotations.Nullable;
import org.spongepowered.asm.mixin.Mixin;

@Mixin(WhiteRectangleGlyph.class)
public class WhiteRectangleGlyphMixin implements NativeImageBackedGlyph {
    @Override
    @Nullable
    public NativeImage getBackedImage() {
        return WhiteRectangleGlyph.IMAGE;
    }
}
