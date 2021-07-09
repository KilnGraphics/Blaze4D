package me.hydos.blaze4d.mixin.integration;

import net.minecraft.client.render.BufferBuilder;
import net.minecraft.client.render.BufferRenderer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(BufferRenderer.class)
public class BufferRendererMixin {

    @Inject(method = "draw(Lnet/minecraft/client/render/BufferBuilder;)V", at = @At("HEAD"), cancellable = true)
    private static void drawConsumer(BufferBuilder bufferBuilder, CallbackInfo ci) {
        bufferBuilder.clear();
        ci.cancel();
    }
}
