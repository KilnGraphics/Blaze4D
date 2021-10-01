package me.hydos.blaze4d.mixin.blaze3d;

import com.mojang.blaze3d.systems.RenderSystem;
import com.mojang.blaze3d.vertex.Tesselator;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import org.lwjgl.glfw.GLFW;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(value = RenderSystem.class, remap = false)
public class RenderSystemMixin {

    @Inject(method = "maxSupportedTextureSize", at = @At("HEAD"), cancellable = true)
    private static void setMaxSupportedTextureSize(CallbackInfoReturnable<Integer> cir) {
        cir.setReturnValue(99999999);
    }

    /**
     * @author Blaze4D
     * @reason Removal Of GL Specific Code
     */
    @Overwrite
    public static void flipFrame(long window) {
        RenderSystem.replayQueue();
        Tesselator.getInstance().getBuilder().clear();
        GLFW.glfwPollEvents();
        GlobalRenderSystem.postDraw();
    }
}
