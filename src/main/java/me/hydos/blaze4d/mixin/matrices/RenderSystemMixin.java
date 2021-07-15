package me.hydos.blaze4d.mixin.matrices;

import com.mojang.blaze3d.systems.RenderSystem;
import com.mojang.math.Matrix4f;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import me.hydos.blaze4d.api.util.ConversionUtils;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(value = RenderSystem.class, remap = false)
public class RenderSystemMixin {
    @Unique
    private static org.joml.Matrix4f savedProjection;

    @Inject(method = "setProjectionMatrix", at = @At("HEAD"), cancellable = true)
    private static void convertAndSetProjection(Matrix4f matrix4f, CallbackInfo ci) {
        GlobalRenderSystem.projectionMatrix = ConversionUtils.mcToJomlProjectionMatrix(matrix4f);
    }

    @Inject(method = "_backupProjectionMatrix", at = @At("HEAD"), cancellable = true)
    private static void backupProjection(CallbackInfo ci) {
        savedProjection = GlobalRenderSystem.projectionMatrix;
    }

    @Inject(method = "_restoreProjectionMatrix", at = @At("HEAD"), cancellable = true)
    private static void restoreProjection(CallbackInfo ci) {
        GlobalRenderSystem.projectionMatrix = savedProjection;
    }
}
