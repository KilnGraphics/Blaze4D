package me.hydos.blaze4d.mixin.matrices;

import com.mojang.blaze3d.systems.RenderSystem;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import net.minecraft.util.math.Matrix4f;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(RenderSystem.class)
public class RenderSystemMixin {
    @Unique
    private static org.joml.Matrix4f savedProjection;

    @Inject(method = "setProjectionMatrix", at = @At("HEAD"), cancellable = true)
    private static void convertAndSetProjection(Matrix4f matrix4f, CallbackInfo ci) {
        // 0 = not confident, 1 = pretty confident, 2 = fully confident
        GlobalRenderSystem.projectionMatrix = new org.joml.Matrix4f(
                matrix4f.a00, //2
                matrix4f.a10, //0
                matrix4f.a20, //0
                matrix4f.a30, //0
                matrix4f.a01, //0
                -matrix4f.a11, //1
                matrix4f.a21, //0
                matrix4f.a31, //0
                matrix4f.a02, //0
                matrix4f.a12, //0
                matrix4f.a22 / 2.0F, //2
                matrix4f.a32, //0
                matrix4f.a03, //2
                -matrix4f.a13, //2
                (matrix4f.a23 + 1) / 2.0F, //2
                matrix4f.a33 //1
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
