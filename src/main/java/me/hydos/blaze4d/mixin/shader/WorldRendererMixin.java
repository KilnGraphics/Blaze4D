package me.hydos.blaze4d.mixin.shader;

import me.hydos.blaze4d.api.GlobalRenderSystem;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import net.minecraft.client.render.RenderLayer;
import net.minecraft.client.render.WorldRenderer;
import net.minecraft.client.util.math.MatrixStack;
import net.minecraft.util.math.Matrix4f;

@Mixin(value = WorldRenderer.class, priority = 1001)
public class WorldRendererMixin {

    @Inject(method = "renderStars()V", at = @At(target = "Lnet/minecraft/client/render/WorldRenderer;renderStars(Lnet/minecraft/client/render/BufferBuilder;)V", value = "INVOKE"), cancellable = true)
    public void cancelRenderStars(CallbackInfo ci) {
        ci.cancel();
    }

    @Inject(method = "renderLightSky", at = @At(target = "Lnet/minecraft/client/render/WorldRenderer;method_34550(Lnet/minecraft/client/render/BufferBuilder;F)V", value = "INVOKE"), cancellable = true)
    public void cancelRenderLightSky(CallbackInfo ci) {
        ci.cancel();
    }

    @Inject(method = "renderDarkSky", at = @At(target = "Lnet/minecraft/client/render/WorldRenderer;method_34550(Lnet/minecraft/client/render/BufferBuilder;F)V", value = "INVOKE"), cancellable = true)
    public void cancelRenderDarkSky(CallbackInfo ci) {
        ci.cancel();
    }

    @Inject(method = "renderLayer", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/gl/GlUniform;set(FFF)V"), require = 0)
    private void redirectChunkOffset(RenderLayer renderLayer, MatrixStack matrices, double x, double y, double z, Matrix4f matrix4f, CallbackInfo ci) {
        //TODO: set the chunk offset so chunks can render properly
        GlobalRenderSystem.chunkOffset.set(x, y, z);
    }
}
