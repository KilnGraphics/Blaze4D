package me.hydos.blaze4d.mixin.integration;

import me.hydos.blaze4d.api.vertex.UploadableConsumer;
import net.minecraft.client.render.BufferBuilder;
import net.minecraft.client.render.BufferRenderer;
import net.minecraft.client.render.Tessellator;
import net.minecraft.client.render.VertexFormat;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.nio.ByteBuffer;

@Mixin(BufferRenderer.class)
public class BufferRendererMixin {

    @Inject(method = "draw(Lnet/minecraft/client/render/BufferBuilder;)V", at = @At("HEAD"), cancellable = true)
    private static void drawConsumer(BufferBuilder bufferBuilder, CallbackInfo ci) {
        bufferBuilder.clear();
        ci.cancel();
    }
}
