package me.hydos.rosella.render.texture;

import java.util.Collection;

public interface TextureMap {
    Texture get(String samplerName);
    Collection<Texture> getTextures();
}
