package me.hydos.rosella;

import me.hydos.rosella.device.VulkanDevice;
import me.hydos.rosella.device.VulkanQueues;
import me.hydos.rosella.display.Display;
import me.hydos.rosella.logging.DebugLogger;
import me.hydos.rosella.logging.DefaultDebugLogger;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.material.PipelineManager;
import me.hydos.rosella.render.object.Renderable;
import me.hydos.rosella.render.renderer.Renderer;
import me.hydos.rosella.render.shader.RawShaderProgram;
import me.hydos.rosella.render.shader.ShaderManager;
import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.swapchain.Frame;
import me.hydos.rosella.render.texture.TextureManager;
import me.hydos.rosella.render.util.memory.Memory;
import me.hydos.rosella.vkobjects.VkCommon;
import me.hydos.rosella.vkobjects.VulkanInstance;
import org.apache.logging.log4j.Level;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.vulkan.VkLayerProperties;

import java.nio.IntBuffer;
import java.util.List;
import java.util.Set;
import java.util.stream.Collectors;

import static me.hydos.rosella.render.util.VkUtilsKt.ok;
import static org.lwjgl.vulkan.KHRSurface.vkDestroySurfaceKHR;
import static org.lwjgl.vulkan.VK10.*;

/**
 * Main Rosella class. If your interacting with the engine from here, You will most likely be safe.
 */
public class Rosella {

    public static final Logger LOGGER = LogManager.getLogger("Rosella");
    public static final int POLYGON_MODE = VK_POLYGON_MODE_FILL;
    public final VkCommon common = new VkCommon();
    public final ShaderManager shaderManager;
    public final TextureManager textureManager;
    public final Memory memory;
    public final Renderer renderer;
    public final PipelineManager pipelineManager;

    public Rosella(Display display, List<String> requestedValidationLayers, String applicationName) {
        this(display, requestedValidationLayers, applicationName, new DefaultDebugLogger());
    }

    public Rosella(Display display, List<String> requestedValidationLayers, String applicationName, DebugLogger debugLogger) {
        List<String> requiredExtensions = display.getRequiredExtensions();
        if (!validationLayersSupported(requestedValidationLayers)) {
            throw new RuntimeException("The application requested validation layers but they are not supported");
        }

        // Setup core vulkan stuff
        common.vkInstance = new VulkanInstance(requestedValidationLayers, requiredExtensions, applicationName, debugLogger);
        common.surface = display.createSurface(common);
        common.device = new VulkanDevice(common, requestedValidationLayers);
        common.queues = new VulkanQueues(common);

        // Setup the engine
        this.shaderManager = new ShaderManager(common);
        this.textureManager = new TextureManager(common);
        this.memory = new Memory(common);
        this.renderer = new Renderer(common); //TODO: make swapchain, etc initialization happen outside of the renderer and in here
        this.pipelineManager = new PipelineManager(common, renderer);

        // Tell the display we are initialized
        display.onReady();
    }

    //=======================//
    //      Scene Stuff      //
    //=======================//

    /**
     * adds an object into the current scene.
     *
     * @param renderable the material to add to the scene
     */
    public void addObject(Renderable renderable) {

    }

    /**
     * registers a {@link Material} into the engine.
     *
     * @param material the material to register
     */
    public void registerMaterial(Material material) {

    }

    /**
     * registers a {@link RawShaderProgram} into the engine.
     *
     * @param program the program to register
     */
    public void addShader(RawShaderProgram program) {

    }

    /**
     * registers a {@link ShaderProgram} into the engine.
     *
     * @param program the program to register
     */
    public void addShader(ShaderProgram program) {

    }

    /**
     * Free's the vulkan resources.
     */
    public void free() {
        waitForIdle();

        // Free the Scene

        // Free Material related stuff
        shaderManager.free();
        renderer.freeSwapChain(memory);
        for (Frame frame : renderer.inFlightFrames) {
            vkDestroySemaphore(common.device.rawDevice, frame.renderFinishedSemaphore(), null);
            vkDestroySemaphore(common.device.rawDevice, frame.imageAvailableSemaphore(), null);
            vkDestroyFence(common.device.rawDevice, frame.fence(), null);
        }

        // Free the rest of it
        vkDestroyCommandPool(common.device.rawDevice, renderer.getCommandPool(), null);
        renderer.swapchain.free(common.device.rawDevice);
        vkDestroyDevice(common.device.rawDevice, null);
        vkDestroySurfaceKHR(common.vkInstance.rawInstance, common.surface, null);
        vkDestroyInstance(common.vkInstance.rawInstance, null);
        memory.free();
    }

    /**
     * Waits for the engine to stop rendering and be idle. any anything that is freed after this is 99% likely to be safe.
     */
    private void waitForIdle() {
        vkDeviceWaitIdle(common.device.rawDevice);
        vkQueueWaitIdle(common.queues.graphicsQueue);
        vkQueueWaitIdle(common.queues.presentQueue);
    }

    /**
     * Checks if the system supports validation layers.
     *
     * @param requestedValidationLayers the validation layers requested by the application/user
     * @return if the system supports the request validation layers.
     */
    private boolean validationLayersSupported(List<String> requestedValidationLayers) {
        return getSupportedValidationLayers().containsAll(requestedValidationLayers);
    }

    /**
     * Gets all validation layers supported by the machine
     *
     * @return all validation layers that are supported
     */
    private Set<String> getSupportedValidationLayers() {
        try (MemoryStack stack = MemoryStack.stackPush()) {
            IntBuffer pLayerCount = stack.ints(0);
            ok(vkEnumerateInstanceLayerProperties(pLayerCount, null));
            VkLayerProperties.Buffer availableLayers = VkLayerProperties.mallocStack(pLayerCount.get(0), stack);
            ok(vkEnumerateInstanceLayerProperties(pLayerCount, availableLayers));
            return availableLayers.stream()
                    .map(VkLayerProperties::layerNameString)
                    .collect(Collectors.toSet());
        }

    }

    static {
        LOGGER.atLevel(Level.ALL);
    }
}
