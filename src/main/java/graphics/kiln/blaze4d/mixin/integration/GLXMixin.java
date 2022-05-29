package graphics.kiln.blaze4d.mixin.integration;

import com.mojang.blaze3d.platform.GLX;
import com.mojang.blaze3d.platform.GlStateManager;
import graphics.kiln.blaze4d.Blaze4D;
import org.lwjgl.glfw.GLFW;
import org.lwjgl.system.APIUtil;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.util.function.LongSupplier;

@Mixin(value = GLX.class, remap = false)
public class GLXMixin {

//
//    @Inject(method = "getOpenGLVersionString", at = @At("HEAD"), cancellable = true)
//    private static void getVulkanInfoString(CallbackInfoReturnable<String> cir) {
//        cir.setReturnValue(Blaze4D.rosella == null || Blaze4D.rosella.common.device == null ? "NO CONTEXT" : GlStateManager._getString(7937) + " " + GlStateManager._getString(7938) + ", " + GlStateManager._getString(7936));
//    }
}
