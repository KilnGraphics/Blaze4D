package me.hydos.rosella.render.texture;

import it.unimi.dsi.fastutil.objects.Object2ObjectOpenHashMap;

import java.util.Collection;
import java.util.Map;

public class DynamicTextureMap implements TextureMap {
    private final Map<String, Texture> map;

    public DynamicTextureMap() {
        this.map = new Object2ObjectOpenHashMap<>();
    }

    public Texture put(String samplerName, Texture texture) {
        return map.put(samplerName, texture);
    }

    public Texture remove(String samplerName) {
        return map.remove(samplerName);
    }

    public void clear() {
        map.clear();
    }

    @Override
    public Texture get(String samplerName) {
        return map.get(samplerName);
    }

    @Override
    public Collection<Texture> getTextures() {
        return map.values();
    }
}
