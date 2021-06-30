package me.hydos.blaze4d.mixin.integration;

import me.hydos.blaze4d.api.GlobalRenderSystem;
import net.minecraft.util.math.Matrix4f;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(Matrix4f.class)
public class Matrix4fMixin {

    @Inject(method = "projectionMatrix(FFFFFF)Lnet/minecraft/util/math/Matrix4f;", at = @At("HEAD"))
    private static void createVkProjectionMatrix(float left, float right, float bottom, float top, float nearPlane, float farPlane, CallbackInfoReturnable<Matrix4f> cir) {
        GlobalRenderSystem.projectionMatrix = new org.joml.Matrix4f().setOrtho(
                left,
                right,
                bottom,
                top,
                nearPlane,
                farPlane,
                true
        );
    }
}
