package me.hydos.blaze4d.mixin.blaze3d;

import com.mojang.blaze3d.systems.RenderSystem;
import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import net.minecraft.client.render.Tessellator;
import org.lwjgl.glfw.GLFW;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(RenderSystem.class)
public class RenderSystemMixin {

    @Inject(method = "initRenderer", remap = false, at = @At("HEAD"))
    private static void cancel_initRenderer(int debugVerbosity, boolean debugSync, CallbackInfo ci) {
    }

    @Inject(method = "maxSupportedTextureSize", remap = false, at = @At("HEAD"), cancellable = true)
    private static void setMaxSupportedTextureSize(CallbackInfoReturnable<Integer> cir) {
        cir.setReturnValue(99999999);
    }

    @Inject(method = "isOnRenderThread",remap = false, at = @At("HEAD"), cancellable = true)
    private static void myEngineIsMultithreadedAndSafe(CallbackInfoReturnable<Boolean> cir) {
        cir.setReturnValue(true);
        // TODO: if something crashes, point out this was here
    }

    /**
     * @author Blaze4D
     * @reason Removal Of GL Specific Code
     */
    @Overwrite
    public static void flipFrame(long window) {
        RenderSystem.replayQueue();
        Tessellator.getInstance().getBuffer().clear();
        GlobalRenderSystem.flipFrame();
        GLFW.glfwPollEvents();
    }
}
