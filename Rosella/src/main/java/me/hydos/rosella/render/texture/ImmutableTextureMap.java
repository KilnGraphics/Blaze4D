package me.hydos.rosella.render.texture;

import it.unimi.dsi.fastutil.objects.Object2ObjectArrayMap;
import it.unimi.dsi.fastutil.objects.Object2ObjectOpenHashMap;
import me.hydos.rosella.Rosella;

import java.util.Collection;
import java.util.Map;

import static org.lwjgl.vulkan.VK10.VK_FORMAT_R8G8B8A8_UNORM;

public class ImmutableTextureMap implements TextureMap {
    private final Map<String, Texture> map;

    public ImmutableTextureMap(Map<String, Texture> backingMap) {
        this.map = backingMap;
    }

    public ImmutableTextureMap(UploadableImage[] images, SamplerCreateInfo samplerCreateInfo, Rosella rosella, TextureManager textureManager) {
        this.map = new Object2ObjectArrayMap<>();
        for (int i = 0; i < images.length; i++) {
            UploadableImage image = images[i];
            if (image == null) {
                this.map.put("texSampler", TextureManager.BLANK_TEXTURE);
                continue;
            }

            int textureId = textureManager.generateTextureId();
            textureManager.createTexture(
                    rosella.renderer,
                    textureId,
                    image.getWidth(),
                    image.getHeight(),
                    VK_FORMAT_R8G8B8A8_UNORM
            );
            textureManager.setTextureSampler(
                    textureId,
                    "texSampler",
                    samplerCreateInfo
            );
            textureManager.drawToExistingTexture(rosella.renderer, textureId, image);

            this.map.put("texSampler", textureManager.getTexture(textureId));
        }
    }

    @Override
    public Texture get(String samplerName) {
        return map.get(samplerName);
    }

    @Override
    public Collection<Texture> getTextures() {
        return map.values();
    }

    public static Builder builder() {
        return new Builder();
    }

    public static class Builder {
        private final Map<String, Texture> map;

        private Builder() {
            map = new Object2ObjectOpenHashMap<>();
        }

        public Builder entry(String samplerName, Texture texture) {
            map.put(samplerName, texture);
            return this;
        }

        public ImmutableTextureMap build() {
            return new ImmutableTextureMap(map);
        }
    }
}
