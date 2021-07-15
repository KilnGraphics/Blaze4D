package me.hydos.blaze4d.mixin.shader;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import com.mojang.blaze3d.shaders.Uniform;
import java.nio.FloatBuffer;
import java.nio.IntBuffer;

@Mixin(Uniform.class)
public abstract class GlUniformMixin {

//    @Redirect(method = "<init>", at = @At(value = "INVOKE", target = "Lorg/lwjgl/system/MemoryUtil;memAllocInt(I)Ljava/nio/IntBuffer;"))
//    private IntBuffer redirectIntAllocation(int size) {
//        return null;
//    }
//
//    @Redirect(method = "<init>", at = @At(value = "INVOKE", target = "Lorg/lwjgl/system/MemoryUtil;memAllocFloat(I)Ljava/nio/FloatBuffer;"))
//    private FloatBuffer redirectFloatAllocation(int size) {
//        return null;
//    }
//
//    @Inject(method = "close", at = @At("HEAD"), cancellable = true)
//    private void noopClose(CallbackInfo ci) {
//        ci.cancel();
//    }
//
//    @Inject(method = {
//            "set(F)V",
//            "set([F)V",
//            "set(FF)V",
//            "set(FFF)V",
//            "set(FFFF)V",
//            "set(FFFFFF)V",
//            "set(FFFFFFFF)V",
//            "set(FFFFFFFFFFFF)V",
//            "set(FFFFFFFFFFFFFFFF)V",
//            "method_35653",
//            "setForDataType(FFFF)V",
//            "set(Lnet/minecraft/util/math/Vec3f;)V",
//            "set(Lnet/minecraft/util/math/Vector4f;)V"
//    }, at = @At(value = "INVOKE", target = "Ljava/nio/FloatBuffer;put(IF)Ljava/nio/FloatBuffer;"), cancellable = true)
//    private void redirectFloatPut(CallbackInfo ci) {
//        ci.cancel();
//    }
}
