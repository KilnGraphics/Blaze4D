package me.hydos.blaze4d.api.texture;

import me.hydos.rosella.render.texture.UploadableImage;
import net.minecraft.client.texture.NativeImage;
import org.joml.Vector2i;

import java.util.HashMap;
import java.util.Map;

public class BlankTextures {
    private static final Map<TextureData, UploadableImage> textureCache = new HashMap<>();

    public static UploadableImage getOrCreateTex(int width, int height, NativeImage.Format format) {
        return textureCache.computeIfAbsent(new TextureData(width, height, format), data -> {
            // all useStb does is calloc, and we want that to initialize all the pixel values to 0
            NativeImage nativeImage = new NativeImage(data.format, data.width, data.height, true);
            nativeImage.untrack();
            return (UploadableImage) (Object) nativeImage;
        });
    }

    public static UploadableImage getOrCreateTex(int width, int height, NativeImage.GLFormat format) {
        return getOrCreateTex(width, height, NativeImage.Format.getFormat(format.getGlConstant()));
    }

    private record TextureData(int width, int height, NativeImage.Format format) {}
}
