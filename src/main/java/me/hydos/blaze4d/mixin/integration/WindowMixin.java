package me.hydos.blaze4d.mixin.integration;

import com.mojang.blaze3d.platform.DisplayData;
import com.mojang.blaze3d.platform.ScreenManager;
import com.mojang.blaze3d.platform.VideoMode;
import com.mojang.blaze3d.platform.WindowEventHandler;
import com.oroarmor.aftermath.Aftermath;
import me.hydos.blaze4d.AftermathHandler;
import me.hydos.blaze4d.Blaze4D;
import graphics.kiln.rosella.Rosella;
import graphics.kiln.rosella.display.GlfwWindow;
import org.apache.logging.log4j.Logger;
import org.lwjgl.glfw.Callbacks;
import org.lwjgl.glfw.GLFW;
import org.lwjgl.glfw.GLFWErrorCallback;
import org.lwjgl.glfw.GLFWImage;
import org.lwjgl.opengl.GL;
import org.lwjgl.opengl.GLCapabilities;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.system.MemoryUtil;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Mutable;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.ModifyArg;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.LocalCapture;

import java.io.InputStream;
import java.nio.ByteBuffer;
import java.nio.IntBuffer;
import java.util.Optional;

import net.fabricmc.loader.api.FabricLoader;

@Mixin(com.mojang.blaze3d.platform.Window.class)
public abstract class WindowMixin {

    @Shadow
    @Final
    private static Logger LOGGER;

    @Shadow
    private Optional<VideoMode> preferredFullscreenVideoMode;

    @Shadow
    private int windowedX;

    @Shadow
    private int windowedY;

    @Shadow
    private int x;

    @Shadow
    private int y;

    @Mutable
    @Shadow
    @Final
    private long window;

    @Shadow
    private int width;

    @Shadow
    private int height;

    @Shadow
    private boolean fullscreen;

    @Shadow
    private int framebufferWidth;

    @Shadow
    private int framebufferHeight;

    @Shadow
    protected abstract void setMode();

    @Shadow
    protected abstract void onMove(long window, int x, int y);

    @Shadow
    protected abstract void onFocus(long window, boolean focused);

    @Shadow
    protected abstract void onEnter(long window, boolean entered);

    @Shadow protected abstract void onResize(long window, int width, int height);

    @Shadow protected abstract void onFramebufferResize(long window, int width, int height);

    @Shadow @Final private GLFWErrorCallback defaultErrorCallback;

    @Inject(method = "bootCrash", at = @At("HEAD"), cancellable = true)
    private static void silenceGl(int error, long description, CallbackInfo ci) {
        String message = "suppressed GLFW/OpenGL error " + error + ": " + MemoryUtil.memUTF8(description);
        LOGGER.warn(message);
        ci.cancel();
    }

    @ModifyArg(method = "<init>", at = @At(value = "INVOKE", target = "Lorg/lwjgl/glfw/GLFW;glfwWindowHint(II)V", ordinal = 0, remap = false), index = 1)
    private int setNoApi(int initialApi) {
        return GLFW.GLFW_NO_API;
    }

    @Redirect(method = "<init>", at = @At(value = "INVOKE", target = "Lorg/lwjgl/opengl/GL;createCapabilities()Lorg/lwjgl/opengl/GLCapabilities;", remap = false))
    private GLCapabilities cancelCreateCapabilities() {
        return null;
    }

    @Inject(method = "<init>", at = @At(value = "TAIL"))
    private void initializeRosellaWindow(WindowEventHandler eventHandler, ScreenManager monitorTracker, DisplayData settings, String videoMode, String title, CallbackInfo ci) {
        Blaze4D.window = new GlfwWindow.SuppliedGlfwWindow(window);
        Blaze4D.rosella = new Rosella(Blaze4D.window, "Blaze4D", Blaze4D.VALIDATION_ENABLED);
        Blaze4D.finishSetup();

        try {
            AftermathHandler.initialize(Thread.currentThread());
        } catch (Throwable throwable) {
            // We don't really care if this doesn't work, especially outside of development
            if (FabricLoader.getInstance().isDevelopmentEnvironment()) {
                throwable.printStackTrace();
            }
        }
    }

    @Inject(method = "onFramebufferResize", at = @At(value = "INVOKE", target = "Lcom/mojang/blaze3d/platform/WindowEventHandler;resizeDisplay()V"))
    private void noticeRenderer(long window, int width, int height, CallbackInfo ci) {
        Blaze4D.rosella.renderer.queueRecreateSwapchain();
    }

    @Inject(method = "updateVsync", at = @At(value = "INVOKE", target = "Lorg/lwjgl/glfw/GLFW;glfwSwapInterval(I)V", remap = false), cancellable = true)
    private void setVsync(boolean vsync, CallbackInfo ci) {
        boolean previousVsync = Blaze4D.window.doVsync;
        if (previousVsync != vsync) {
            Blaze4D.window.doVsync = vsync;
            Blaze4D.rosella.renderer.queueRecreateSwapchain(); // TODO: move this probably
        }
        ci.cancel();
    }

    @Inject(method = "setIcon", at = @At(value = "INVOKE", target = "Lorg/lwjgl/glfw/GLFW;glfwSetWindowIcon(JLorg/lwjgl/glfw/GLFWImage$Buffer;)V", remap = false), locals = LocalCapture.CAPTURE_FAILSOFT)
    private void setIcon(InputStream icon16, InputStream icon32, CallbackInfo ci, MemoryStack memoryStack, IntBuffer intBuffer, IntBuffer intBuffer2, IntBuffer intBuffer3, GLFWImage.Buffer buffer, ByteBuffer byteBuffer, ByteBuffer byteBuffer2) {
        GLFW.glfwSetWindowIcon(window, buffer);
    }

    @Inject(method = "close", at = @At("HEAD"), cancellable = true)
    private void freeRosella(CallbackInfo ci) {
        Callbacks.glfwFreeCallbacks(this.window);
//        this.defaultErrorCallback.close();

        if (Blaze4D.rosella != null) {
            Blaze4D.rosella.free();
            Blaze4D.rosella = null;
        }
        Aftermath.disableGPUCrashDumps();
        ci.cancel();
    }
}
