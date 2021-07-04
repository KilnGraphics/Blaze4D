package me.hydos.rosella.display;

import me.hydos.rosella.vkobjects.VkCommon;
import org.lwjgl.PointerBuffer;
import org.lwjgl.glfw.GLFWVidMode;
import org.lwjgl.glfw.GLFWVulkan;
import org.lwjgl.system.MemoryStack;

import java.nio.LongBuffer;
import java.util.List;

import static me.hydos.rosella.render.util.VkUtilsKt.ok;
import static org.lwjgl.glfw.GLFW.*;
import static org.lwjgl.glfw.GLFWVulkan.glfwCreateWindowSurface;
import static org.lwjgl.vulkan.VK10.VK_NULL_HANDLE;

/**
 * An implementation of {@link Display} using GLFW.
 */
public class GlfwWindow extends Display {

    public final long pWindow;

    // Fps Stuff
    public double previousTime = glfwGetTime();
    public int frameCount;

    public GlfwWindow(int width, int height, String title, boolean canResize) {
        super(width, height);

        if (!GLFWVulkan.glfwVulkanSupported()) {
            throw new RuntimeException("Your machine doesn't support Vulkan :(");
        }

        if (!glfwInit()) {
            throw new RuntimeException("Failed to Initialize GLFW");
        }
        glfwWindowHint(GLFW_CLIENT_API, GLFW_NO_API);
        glfwWindowHint(GLFW_VISIBLE, GLFW_FALSE);
        glfwWindowHint(GLFW_RESIZABLE, canResize ? GLFW_TRUE : GLFW_FALSE);
        pWindow = glfwCreateWindow(width, height, title, 0, 0);
    }

    /**
     * Retrieves the {@link GLFWVidMode} for the display the window is currently active on.
     *
     * @return the window's {@link GLFWVidMode}
     */
    public GLFWVidMode getVideoMode() {
        return glfwGetVideoMode(glfwGetWindowMonitor(pWindow));
    }

    /**
     * Updates the title displayed generally on top of the window.
     *
     * @param title The string to set it to.
     */
    public void updateTitle(String title) {
        glfwSetWindowTitle(pWindow, title);
    }

    @Override
    public void update() {
        super.update();
        glfwPollEvents();
    }

    @Override
    public void startAutomaticLoop() {
        while (!glfwWindowShouldClose(pWindow)) {
            update();
        }
    }

    @Override
    public void exit() {
        glfwDestroyWindow(pWindow);
        glfwTerminate();
    }

    @Override
    public List<String> getRequiredExtensions() {
        PointerBuffer requiredExtensions = GLFWVulkan.glfwGetRequiredInstanceExtensions();
        throw new RuntimeException("Not Implemented!");
//        return requestedValidationLayers;
    }

    @Override
    public long createSurface(VkCommon common) {
        try (MemoryStack stack = MemoryStack.stackPush()) {
            LongBuffer pSurface = stack.longs(VK_NULL_HANDLE);
            ok(glfwCreateWindowSurface(common.vkInstance.rawInstance, pWindow, null, pSurface));
            return pSurface.get(0);
        }
    }

    @Override
    protected void calculateFps() {
        double currentTime = glfwGetTime();
        frameCount++;
        if (currentTime - previousTime >= 1.0) {
            fps = frameCount;

            frameCount = 0;
            previousTime = currentTime;
        }
    }

    @Override
    public void onReady() {
        glfwShowWindow(pWindow);
    }
}
