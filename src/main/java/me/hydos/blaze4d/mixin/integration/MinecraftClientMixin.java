package me.hydos.blaze4d.mixin.integration;

import me.hydos.blaze4d.api.GlobalRenderSystem;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.render.Tessellator;

import me.hydos.blaze4d.api.vertex.UploadableConsumer;
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
}
