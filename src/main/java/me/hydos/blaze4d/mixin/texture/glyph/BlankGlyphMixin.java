package me.hydos.blaze4d.mixin.texture.glyph;

import me.hydos.blaze4d.api.texture.glyph.NativeImageBackedGlyph;
import net.minecraft.client.font.BlankGlyph;
import net.minecraft.client.texture.NativeImage;
import org.jetbrains.annotations.Nullable;
import org.spongepowered.asm.mixin.Mixin;

@Mixin(BlankGlyph.class)
public class BlankGlyphMixin implements NativeImageBackedGlyph {
    @Override
    @Nullable
    public NativeImage getBackedImage() {
        return BlankGlyph.IMAGE;
    }
}
