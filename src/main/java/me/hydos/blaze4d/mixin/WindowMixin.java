package me.hydos.blaze4d.mixin;

import me.hydos.blaze4d.Blaze4D;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.render.io.Window;
import net.minecraft.client.WindowEventHandler;
import net.minecraft.client.WindowSettings;
import net.minecraft.client.util.MonitorTracker;
import org.lwjgl.glfw.GLFW;
import org.lwjgl.glfw.GLFWImage;
import org.lwjgl.system.MemoryStack;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.LocalCapture;

import java.io.InputStream;
import java.nio.ByteBuffer;
import java.nio.IntBuffer;

@Mixin(net.minecraft.client.util.Window.class)
public class WindowMixin {

    @Inject(method = "<init>", at = @At("TAIL"))
    private void initializeRosellaWindow(WindowEventHandler eventHandler, MonitorTracker monitorTracker, WindowSettings settings, String videoMode, String title, CallbackInfo ci) {
        Blaze4D.window = new Window(title, settings.width, settings.height, true);
        Blaze4D.rosella = new Rosella("Blaze4D", true, Blaze4D.window);
        Blaze4D.prepare();
        Blaze4D.finishAndRender();
    }

    @Inject(method = "setIcon", at = @At(value = "INVOKE", target = "Lorg/lwjgl/glfw/GLFW;glfwSetWindowIcon(JLorg/lwjgl/glfw/GLFWImage$Buffer;)V"), locals = LocalCapture.CAPTURE_FAILSOFT)
    private void setIcon(InputStream icon16, InputStream icon32, CallbackInfo ci, MemoryStack memoryStack, IntBuffer intBuffer, IntBuffer intBuffer2, IntBuffer intBuffer3, GLFWImage.Buffer buffer, ByteBuffer byteBuffer, ByteBuffer byteBuffer2) {
        GLFW.glfwSetWindowIcon(Blaze4D.window.getWindowPtr(), buffer);
    }
}
