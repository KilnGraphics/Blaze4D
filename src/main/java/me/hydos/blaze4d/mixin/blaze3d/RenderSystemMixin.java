package me.hydos.blaze4d.mixin.blaze3d;

import com.mojang.blaze3d.systems.RenderSystem;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(RenderSystem.class)
public class RenderSystemMixin {

    @Inject(method = "initRenderer", at = @At("HEAD"))
    private static void cancel_initRenderer(int debugVerbosity, boolean debugSync, CallbackInfo ci) {
    }

    @Inject(method = "maxSupportedTextureSize", at = @At("HEAD"), cancellable = true)
    private static void setMaxSupportedTextureSize(CallbackInfoReturnable<Integer> cir) {
        cir.setReturnValue(99999999);
    }

    @Inject(method = "isOnRenderThread", at = @At("HEAD"), cancellable = true)
    private static void myEngineIsMultithreadedAndSafe(CallbackInfoReturnable<Boolean> cir) {
        cir.setReturnValue(true);
        // TODO: if something crashes, point out this was here
    }
}
