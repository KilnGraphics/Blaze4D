package me.hydos.blaze4d.mixin.integration;

import me.hydos.blaze4d.api.GlobalRenderSystem;
import me.hydos.blaze4d.api.shader.MinecraftUbo;
import net.minecraft.util.math.Matrix4f;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(Matrix4f.class)
public class Matrix4fMixin {

    @Inject(method = "projectionMatrix(FFFFFF)Lnet/minecraft/util/math/Matrix4f;", at = @At("HEAD"), cancellable = true)
    private static void createVkProjectionMatrix(float left, float right, float bottom, float top, float nearPlane, float farPlane, CallbackInfoReturnable<Matrix4f> cir) {
        Matrix4f proj = new Matrix4f();
        float x = right - left;
        float y = bottom + top;
        float focalLength = farPlane - nearPlane;
        proj.a00 = 2.0F / x;
        proj.a11 = 2.0F / y;
        proj.a22 = -2.0F / focalLength;
        proj.a03 = -(right + left) / x;
        proj.a13 = -(bottom + top) / y;
        proj.a23 = -(farPlane + nearPlane) / focalLength;
        proj.a33 = 1.0F;
        GlobalRenderSystem.projectionMatrix = MinecraftUbo.toJoml(proj);
        cir.setReturnValue(proj);
    }
}
