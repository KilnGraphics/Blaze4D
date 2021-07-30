package me.hydos.rosella.render.texture;

import it.unimi.dsi.fastutil.objects.Object2ObjectOpenHashMap;

import java.util.Collection;
import java.util.Map;

public class ImmutableTextureMap implements TextureMap {
    private final Map<String, Texture> map;

    public ImmutableTextureMap(Map<String, Texture> backingMap) {
        this.map = backingMap;
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
