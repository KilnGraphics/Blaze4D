package me.hydos.blaze4d.mixin.matrices;

import com.mojang.blaze3d.systems.RenderSystem;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import net.minecraft.util.math.Matrix4f;
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
        GlobalRenderSystem.projectionMatrix = new org.joml.Matrix4f(
                matrix4f.a00,
                matrix4f.a10,
                matrix4f.a20,
                matrix4f.a30,
                matrix4f.a01,
                -matrix4f.a11,
                matrix4f.a21,
                matrix4f.a31,
                matrix4f.a02,
                matrix4f.a12,
                matrix4f.a22 / 2.0F,
                matrix4f.a32,
                matrix4f.a03,
                -matrix4f.a13,
                matrix4f.a23 / 2.0F,
                matrix4f.a33
        );
        ci.cancel();
    }

    @Inject(method = "_backupProjectionMatrix", at = @At("HEAD"), cancellable = true)
    private static void backupProjection(CallbackInfo ci) {
        savedProjection = GlobalRenderSystem.projectionMatrix;
        ci.cancel();
    }

    @Inject(method = "_restoreProjectionMatrix", at = @At("HEAD"), cancellable = true)
    private static void restoreProjection(CallbackInfo ci) {
        GlobalRenderSystem.projectionMatrix = savedProjection;
        ci.cancel();
    }
}
