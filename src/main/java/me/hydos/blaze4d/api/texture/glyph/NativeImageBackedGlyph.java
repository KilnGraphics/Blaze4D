package me.hydos.blaze4d.api.texture.glyph;

import net.minecraft.client.texture.NativeImage;
import org.jetbrains.annotations.Nullable;

public interface NativeImageBackedGlyph {
    @Nullable NativeImage getBackedImage();
}
