package me.hydos.blaze4d.mixin.texture;

import net.minecraft.client.texture.AbstractTexture;
import net.minecraft.client.texture.Sprite;
import net.minecraft.client.texture.SpriteAtlasTexture;
import net.minecraft.client.texture.TextureTickListener;
import net.minecraft.resource.ResourceManager;
import net.minecraft.util.Identifier;
import org.apache.logging.log4j.Logger;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;

import java.io.IOException;
import java.util.List;
import java.util.Map;
import java.util.Set;

@Mixin(SpriteAtlasTexture.class)
public abstract class SpriteAtlasTextureMixin extends AbstractTexture implements TextureTickListener{

    @Shadow @Final private Identifier id;

    @Shadow @Final private List<TextureTickListener> animatedSprites;

    @Shadow @Final private Map<Identifier, Sprite> sprites;

    @Shadow @Final private Set<Identifier> spritesToLoad;

    @Shadow @Final private static Logger LOGGER;

    @Shadow public abstract void clear();

    @Override
    public void load(ResourceManager manager) throws IOException {
    }

    @Override
    public void tick() {
    }
}
