package graphics.kiln.blaze4d.mixin.integration;

import graphics.kiln.blaze4d.Blaze4D;
import graphics.kiln.blaze4d.api.Blaze4DCore;
import net.minecraft.client.Minecraft;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(Minecraft.class)
public class MinecraftClientMixin {
    @Inject(method = "runTick", at = @At(value = "HEAD"))
    private void startFrame(CallbackInfo ci) {
        Blaze4D.core.startFrame();
    }

    @Inject(method = "runTick", at = @At(value = "INVOKE", target = "Lcom/mojang/blaze3d/platform/Window;updateDisplay()V"))
    private void postRenderFrame(boolean renderLevel, CallbackInfo ci) {
        Blaze4D.core.endFrame();
    }
}
