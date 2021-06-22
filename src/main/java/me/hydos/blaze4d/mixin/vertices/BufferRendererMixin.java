package me.hydos.blaze4d.mixin.vertices;

import me.hydos.blaze4d.api.vertex.UploadableConsumer;
import net.minecraft.client.render.BufferBuilder;
import net.minecraft.client.render.BufferRenderer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(BufferRenderer.class)
public class BufferRendererMixin {

    @Inject(method = "draw(Lnet/minecraft/client/render/BufferBuilder;)V", at = @At(value = "HEAD"), cancellable = true)
    private static void redirectDraw(BufferBuilder builder, CallbackInfo ci) {
        ((UploadableConsumer) builder).draw();
    }
}
