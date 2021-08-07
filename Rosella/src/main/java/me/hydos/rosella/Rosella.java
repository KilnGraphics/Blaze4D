package me.hydos.rosella;

import me.hydos.rosella.device.LegacyVulkanDevice;
import me.hydos.rosella.device.VulkanDevice;
import me.hydos.rosella.device.VulkanQueues;
import me.hydos.rosella.display.Display;
import me.hydos.rosella.device.init.DeviceBuilder;
import me.hydos.rosella.device.init.InitializationRegistry;
import me.hydos.rosella.device.init.InstanceBuilder;
import me.hydos.rosella.device.init.VulkanInstance;
import me.hydos.rosella.device.init.features.RosellaLegacy;
import me.hydos.rosella.logging.DebugLogger;
import me.hydos.rosella.logging.DefaultDebugLogger;
import me.hydos.rosella.memory.ThreadPoolMemory;
import me.hydos.rosella.memory.buffer.GlobalBufferManager;
import me.hydos.rosella.render.renderer.Renderer;
import me.hydos.rosella.scene.object.ObjectManager;
import me.hydos.rosella.scene.object.impl.SimpleObjectManager;
import me.hydos.rosella.util.SemaphorePool;
import me.hydos.rosella.vkobjects.VkCommon;
import me.hydos.rosella.vkobjects.LegacyVulkanInstance;
import org.apache.logging.log4j.Level;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;
import org.apache.logging.log4j.message.StringFormatterMessageFactory;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.vulkan.VK10;
import org.lwjgl.vulkan.VkLayerProperties;

import java.nio.IntBuffer;
import java.util.Collections;
import java.util.List;
import java.util.Set;
import java.util.stream.Collectors;

import static me.hydos.rosella.util.VkUtils.ok;
import static org.lwjgl.vulkan.EXTDebugUtils.vkDestroyDebugUtilsMessengerEXT;
import static org.lwjgl.vulkan.KHRSurface.vkDestroySurfaceKHR;
import static org.lwjgl.vulkan.VK10.*;
import static org.lwjgl.vulkan.VK12.VK_API_VERSION_1_2;

/**
 * Main Rosella class. If you're interacting with the engine from here, You will most likely be safe.
 */
public class Rosella {

    public static final Logger LOGGER = LogManager.getLogger("Rosella", new StringFormatterMessageFactory());
    public static final int VULKAN_VERSION = VK_API_VERSION_1_2;
    public final GlobalBufferManager bufferManager;
    public final VkCommon common = new VkCommon();
    public final Renderer renderer;
    public final ObjectManager objectManager;

    public final VulkanInstance vulkanInstance;
    public final VulkanDevice vulkanDevice;

    public Rosella(InitializationRegistry registry, Display display, String applicationName, int applicationVersion) {
        // TODO remove
        registry.enableValidation(true);

        common.display = display;
        display.getRequiredExtensions().forEach(registry::addRequiredInstanceExtensions);

        // Needed because debug callbacks are handled by LegacyVulkanInstance. TODO remove this
        common.vkInstance = new LegacyVulkanInstance(registry, applicationName, applicationVersion, new DefaultDebugLogger());
        this.vulkanInstance = common.vkInstance.newInstance;

        common.surface = display.createSurface(common);
        registry.registerApplicationFeature(new RosellaLegacy(common));
        registry.addRequiredApplicationFeature(RosellaLegacy.NAME);

        this.vulkanDevice = new DeviceBuilder(this.vulkanInstance, registry).build();
        common.device = new LegacyVulkanDevice(this.vulkanDevice);

        RosellaLegacy.RosellaLegacyFeatures legacyFeatures = RosellaLegacy.getMetadata(this.vulkanDevice);
        try {
            common.queues = new VulkanQueues(legacyFeatures.graphicsQueue().get(), legacyFeatures.presentQueue().get());
        } catch (Exception ex) {
            throw new RuntimeException("Not good stuff.");
        }

        // TODO: Tons and tons of old code. Need to remove
        common.memory = new ThreadPoolMemory(common);
        common.semaphorePool = new SemaphorePool(common.device.rawDevice);

        this.objectManager = new SimpleObjectManager(this, common);
        this.renderer = new Renderer(this);
        ((SimpleObjectManager) objectManager).textureManager.initializeBlankTexture(renderer);
        this.objectManager.postInit(renderer);
        this.bufferManager = new GlobalBufferManager(this);

        display.onReady();
    }

    @Deprecated
    public Rosella(Display display, String applicationName, boolean enableBasicValidation) {
        this(display, enableBasicValidation ? Collections.singletonList("VK_LAYER_KHRONOS_validation") : Collections.emptyList(), applicationName, new DefaultDebugLogger());
    }

    @Deprecated
    public Rosella(Display display, List<String> requestedValidationLayers, String applicationName, DebugLogger debugLogger) {
        List<String> requiredExtensions = display.getRequiredExtensions();

        InitializationRegistry initializationRegistry = new InitializationRegistry();
        requestedValidationLayers.forEach(initializationRegistry::addRequiredInstanceLayer);
        requiredExtensions.forEach(initializationRegistry::addRequiredInstanceExtensions);

        // Setup core vulkan stuff
        common.display = display;
        common.vkInstance = new LegacyVulkanInstance(initializationRegistry, applicationName, VK10.VK_MAKE_VERSION(1, 0, 0), debugLogger);

        common.surface = display.createSurface(common);
        initializationRegistry.registerApplicationFeature(new RosellaLegacy(common));

        common.device = new LegacyVulkanDevice(common.vkInstance.newInstance, initializationRegistry);

        RosellaLegacy.RosellaLegacyFeatures legacyFeatures = RosellaLegacy.getMetadata(common.device.newDevice);
        try {
            common.queues = new VulkanQueues(legacyFeatures.graphicsQueue().get(), legacyFeatures.presentQueue().get());
        } catch (Exception ex) {
            throw new RuntimeException("Not good stuff.");
        }

        common.memory = new ThreadPoolMemory(common);
        common.semaphorePool = new SemaphorePool(common.device.rawDevice);

        // Setup the object manager
        this.objectManager = new SimpleObjectManager(this, common);
        this.renderer = new Renderer(this); //TODO: make swapchain, etc initialization happen outside of the renderer and in here
        ((SimpleObjectManager) objectManager).textureManager.initializeBlankTexture(renderer); // TODO: move this maybe
        this.objectManager.postInit(renderer);
        this.bufferManager = new GlobalBufferManager(this);

        // Tell the display we are initialized
        display.onReady();

        this.vulkanInstance = common.vkInstance.newInstance;
        this.vulkanDevice = common.device.newDevice;
    }

    /**
     * Free's the vulkan resources.
     */
    public void free() {
        common.device.waitForIdle();
        objectManager.free();
        renderer.free();

        // Free the rest of it
        common.memory.free();
        common.semaphorePool.free();

        vkDestroyCommandPool(common.device.rawDevice, renderer.commandPool, null);

        vulkanDevice.destroy();

        vkDestroySurfaceKHR(common.vkInstance.rawInstance, common.surface, null);

        common.vkInstance.messenger.ifPresent(messenger -> { // FIXME
            vkDestroyDebugUtilsMessengerEXT(common.vkInstance.rawInstance, messenger, null);
        });

        vulkanInstance.destroy();

        common.display.exit();
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
