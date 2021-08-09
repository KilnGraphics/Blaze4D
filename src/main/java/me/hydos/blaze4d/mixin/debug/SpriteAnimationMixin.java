package me.hydos.blaze4d.mixin.debug;

import net.minecraft.client.renderer.texture.TextureAtlasSprite;
import net.minecraft.client.renderer.texture.Tickable;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(TextureAtlasSprite.AnimatedTexture.class)
public abstract class SpriteAnimationMixin implements Tickable, AutoCloseable {

    @Override
    public void tick() {
        // Nope
    }
}
