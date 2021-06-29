package me.hydos.blaze4d.mixin.integration;

import me.hydos.blaze4d.Blaze4D;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.render.io.Window;
import net.minecraft.client.WindowEventHandler;
import net.minecraft.client.WindowSettings;
import net.minecraft.client.util.Monitor;
import net.minecraft.client.util.MonitorTracker;
import net.minecraft.client.util.VideoMode;
import org.apache.logging.log4j.Logger;
import org.lwjgl.glfw.GLFW;
import org.lwjgl.glfw.GLFWImage;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.system.MemoryUtil;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Mutable;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.LocalCapture;

import java.io.InputStream;
import java.nio.ByteBuffer;
import java.nio.IntBuffer;
import java.util.Optional;

@Mixin(net.minecraft.client.util.Window.class)
public abstract class WindowMixin {

    @Shadow
    private Optional<VideoMode> videoMode;

    @Shadow
    private int windowedX;

    @Shadow
    private int windowedY;

    @Shadow
    private int x;

    @Shadow
    private int y;

    @Shadow
    protected abstract void updateWindowRegion();

    @Mutable
    @Shadow
    @Final
    private long handle;

    @Shadow
    protected abstract void onWindowPosChanged(long window, int x, int y);

    @Shadow
    protected abstract void onWindowFocusChanged(long window, boolean focused);

    @Shadow
    protected abstract void onCursorEnterChanged(long window, boolean entered);

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
    @Final
    private static Logger LOGGER;

    @Inject(method = "<init>", at = @At("TAIL"))
    private void initializeRosellaWindow(WindowEventHandler eventHandler, MonitorTracker monitorTracker, WindowSettings settings, String videoMode, String title, CallbackInfo ci) {
        // Destroy The OpenGL Window before Minecraft Gets Too Attached
        GLFW.glfwDestroyWindow(this.handle);

        Blaze4D.window = new Window(title, this.width, this.height, true);
        Blaze4D.rosella = new Rosella("Blaze4D", Blaze4D.VALIDATION_ENABLED, Blaze4D.window);
        Blaze4D.finishAndRender();

        Monitor monitor = monitorTracker.getMonitor(GLFW.glfwGetPrimaryMonitor());
        this.handle = Blaze4D.window.getWindowPtr();
        if (monitor != null) {
            VideoMode videoMode2 = monitor.findClosestVideoMode(this.fullscreen ? this.videoMode : Optional.empty());
            this.windowedX = this.x = monitor.getViewportX() + videoMode2.getWidth() / 2 - this.width / 2;
            this.windowedY = this.y = monitor.getViewportY() + videoMode2.getHeight() / 2 - this.height / 2;
        } else {
            int[] is = new int[1];
            int[] js = new int[1];
            GLFW.glfwGetWindowPos(this.handle, is, js);
            this.windowedX = this.x = is[0];
            this.windowedY = this.y = js[0];
        }

        this.framebufferWidth = this.width;
        this.framebufferHeight = this.height;

        this.updateWindowRegion();
        GLFW.glfwSetWindowPosCallback(this.handle, this::onWindowPosChanged);
        GLFW.glfwSetWindowFocusCallback(this.handle, this::onWindowFocusChanged);
        GLFW.glfwSetCursorEnterCallback(this.handle, this::onCursorEnterChanged);
    }

    @Inject(method = "setIcon", at = @At(value = "INVOKE", target = "Lorg/lwjgl/glfw/GLFW;glfwSetWindowIcon(JLorg/lwjgl/glfw/GLFWImage$Buffer;)V"), locals = LocalCapture.CAPTURE_FAILSOFT)
    private void setIcon(InputStream icon16, InputStream icon32, CallbackInfo ci, MemoryStack memoryStack, IntBuffer intBuffer, IntBuffer intBuffer2, IntBuffer intBuffer3, GLFWImage.Buffer buffer, ByteBuffer byteBuffer, ByteBuffer byteBuffer2) {
        GLFW.glfwSetWindowIcon(Blaze4D.window.getWindowPtr(), buffer);
    }

    @Inject(method = "throwGlError", at = @At("HEAD"), cancellable = true)
    private static void silenceGl(int error, long description, CallbackInfo ci) {
        String message = "suppressed GLFW/OpenGL error " + error + ": " + MemoryUtil.memUTF8(description);
        LOGGER.warn(message);
    }

    @Inject(method = "close", at = @At("HEAD"))
    private void freeRosella(CallbackInfo ci) {
        if(Blaze4D.rosella != null) {
            Blaze4D.rosella.free();
        }
    }
}
