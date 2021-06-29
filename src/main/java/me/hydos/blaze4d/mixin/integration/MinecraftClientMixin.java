package me.hydos.blaze4d.mixin.integration;

import me.hydos.blaze4d.api.GlobalRenderSystem;
import net.minecraft.client.MinecraftClient;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(MinecraftClient.class)
public class MinecraftClientMixin {

    @Inject(method = "render", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/gl/Framebuffer;endWrite()V"))
    private void renderFrame(boolean tick, CallbackInfo ci) {
        GlobalRenderSystem.render();
    }

    @Inject(method = "render", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/gl/Framebuffer;beginWrite(Z)V"))
    private void captureRenderObjects(boolean tick, CallbackInfo ci) {
        GlobalRenderSystem.beginCaptureRenderObjects();
    }
}
