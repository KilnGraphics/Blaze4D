package me.hydos.blaze4d.mixin.vertices;

import net.minecraft.client.render.BufferBuilder;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

@Mixin(BufferBuilder.class)
public interface BufferBuilderAccessor {

    @Accessor("nextDrawStart")
    int blaze4d$getNextDrawStart();
}
