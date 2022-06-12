package graphics.kiln.blaze4d.mixin.integration;

import graphics.kiln.blaze4d.Blaze4D;
import net.minecraft.client.Minecraft;
import org.lwjgl.glfw.GLFW;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(Minecraft.class)
public class MinecraftClientMixin {
    @Inject(method = "runTick", at = @At(value = "HEAD"))
    private void startFrame(CallbackInfo ci) {
        if (Blaze4D.currentFrame != null) {
            throw new RuntimeException("Started new frame with running old frame");
        } else {
            int[] width = new int[1];
            int[] height = new int[1];
            GLFW.glfwGetWindowSize(Blaze4D.glfwWindow, width, height);
            Blaze4D.currentFrame = Blaze4D.core.startFrame(width[0], height[0]);
        }
    }

    @Inject(method = "runTick", at = @At(value = "INVOKE", target = "Lcom/mojang/blaze3d/platform/Window;updateDisplay()V"))
    private void postRenderFrame(boolean renderLevel, CallbackInfo ci) {
        if (Blaze4D.currentFrame != null) {
            try {
                Blaze4D.currentFrame.close();
            } catch (Exception ex) {
                throw new RuntimeException("Failed to end frame", ex);
            }
            Blaze4D.currentFrame = null;
        }
    }
}
