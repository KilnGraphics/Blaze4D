package graphics.kiln.blaze4d.mixin.integration;

import graphics.kiln.blaze4d.Blaze4D;
import net.minecraft.client.Minecraft;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(Minecraft.class)
public class MinecraftClientMixin {
//    @Inject(method = "runTick", at = @At(value = "INVOKE", target = "Lcom/mojang/blaze3d/pipeline/RenderTarget;unbindWrite()V"))
//    private void renderFrame(boolean tick, CallbackInfo ci) {
//        Blaze4D.core.start_frame();
//    }
//
    @Inject(method = "runTick", at = @At(value = "INVOKE", target = "Lcom/mojang/blaze3d/platform/Window;updateDisplay()V"))
    private void postRenderFrame(boolean renderLevel, CallbackInfo ci) {
        Blaze4D.core.start_frame();
    }
}
