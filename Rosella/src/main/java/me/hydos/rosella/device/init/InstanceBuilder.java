package me.hydos.rosella.device.init;

import me.hydos.rosella.debug.MessageSeverity;
import me.hydos.rosella.debug.MessageType;
import me.hydos.rosella.debug.VulkanDebugCallback;
import me.hydos.rosella.util.VkUtils;
import org.lwjgl.PointerBuffer;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.vulkan.*;

import java.nio.IntBuffer;
import java.util.HashSet;
import java.util.Set;

import static org.lwjgl.vulkan.EXTDebugUtils.*;

public class InstanceBuilder {

    private final InitializationRegistry registry;

    private final boolean enableDebugUtils;
    private final boolean enableValidation;

    private final VulkanDebugCallback debugUtilsCallback;

    public InstanceBuilder(InitializationRegistry registry) {
        this.registry = registry;

        this.enableValidation = registry.getEnableValidation();
        if(!registry.getDebugCallbacks().isEmpty() || registry.getEnableValidation()) {
            this.enableDebugUtils = true;
            this.debugUtilsCallback = new VulkanDebugCallback();

            registry.getDebugCallbacks().forEach(this.debugUtilsCallback::registerCallback);

        } else {
            this.enableDebugUtils = false;
            this.debugUtilsCallback = null;
        }
    }

    public VulkanInstance build(String applicationName, int applicationVersion) {
        try(MemoryStack stack = MemoryStack.stackPush()) {
            int supportedVersionNumber = getSupportedVersion(stack);
            if(supportedVersionNumber < this.registry.getMinimumVulkanVersion().getVersionNumber()) {
                throw new RuntimeException("Minimum vulkan version " + this.registry.getMinimumVulkanVersion().toString() + " is not supported!");
            }

            VkApplicationInfo appInfo = VkApplicationInfo.callocStack(stack);
            appInfo.sType(VK10.VK_STRUCTURE_TYPE_APPLICATION_INFO);
            appInfo.pApplicationName(stack.UTF8(applicationName));
            appInfo.applicationVersion(applicationVersion);
            appInfo.pEngineName(stack.UTF8("Rosella"));
            appInfo.engineVersion(VK10.VK_MAKE_VERSION(0, 1, 0)); // TODO
            appInfo.apiVersion(this.registry.getMaxSupportedVersion().getVersionNumber());

            VkInstanceCreateInfo createInfo = VkInstanceCreateInfo.callocStack(stack);
            createInfo.sType(VK10.VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO);
            createInfo.pNext(createDebugUtilsCallback(VK10.VK_NULL_HANDLE, stack));
            createInfo.pApplicationInfo(appInfo);
            createInfo.ppEnabledLayerNames(getLayerBuffer(stack));
            createInfo.ppEnabledExtensionNames(getExtensionBuffer(stack));

            PointerBuffer pInstance = stack.mallocPointer(1);
            VkUtils.ok(VK10.vkCreateInstance(createInfo, null, pInstance));

            VkInstance instance = new VkInstance(pInstance.get(0), createInfo);
            return new VulkanInstance(instance, this.debugUtilsCallback);
        }
    }

    private int getSupportedVersion(MemoryStack stack) {
        IntBuffer versionBuffer = stack.mallocInt(1);

        try { // Yes this is ugly but i have no better plan to validate that the function actually exists
            VK11.vkEnumerateInstanceVersion(versionBuffer);
        } catch (NullPointerException ex) {
            return VulkanVersion.VULKAN_1_0.getVersionNumber();
        }

        return versionBuffer.get(0);
    }

    private PointerBuffer getLayerBuffer(MemoryStack stack) {
        IntBuffer count = stack.mallocInt(1);
        VkUtils.ok(VK10.vkEnumerateInstanceLayerProperties(count, null));
        VkLayerProperties.Buffer propertiesBuffer = VkLayerProperties.mallocStack(count.get(0), stack);
        VkUtils.ok(VK10.vkEnumerateInstanceLayerProperties(count, propertiesBuffer));

        Set<String> supportedLayers = new HashSet<>();
        for(int i = 0; i < count.get(0); i++) {
            supportedLayers.add(propertiesBuffer.get(i).layerNameString());
        }

        if(!supportedLayers.containsAll(this.registry.getRequiredInstanceLayers())) {
            throw new RuntimeException("Required instance layers not found");
        }

        Set<String> enabledLayers = new HashSet<>();
        if(this.enableValidation) {
            if(!supportedLayers.contains("VK_LAYER_KHRONOS_validation")) {
                throw new RuntimeException("Debug was enabled but validation layers could not be found");
            }

            enabledLayers.add("VK_LAYER_KHRONOS_validation");
        }

        enabledLayers.addAll(this.registry.getRequiredInstanceLayers());
        enabledLayers.addAll(this.registry.getOptionalInstanceLayers().stream().filter(supportedLayers::contains).toList());

        PointerBuffer layerNames = stack.mallocPointer(enabledLayers.size());
        for(String layer : enabledLayers) {
            layerNames.put(stack.UTF8(layer));
        }

        return layerNames.rewind();
    }

    private PointerBuffer getExtensionBuffer(MemoryStack stack) {
        IntBuffer count = stack.mallocInt(1);
        VkUtils.ok(VK10.vkEnumerateInstanceExtensionProperties((CharSequence) null, count, null));
        VkExtensionProperties.Buffer propertiesBuffer = VkExtensionProperties.mallocStack(count.get(0), stack);
        VkUtils.ok(VK10.vkEnumerateInstanceExtensionProperties((CharSequence) null, count, propertiesBuffer));

        Set<String> supportedExtensions = new HashSet<>();
        for(int i = 0; i < count.get(0); i++) {
            supportedExtensions.add(propertiesBuffer.get(i).extensionNameString());
        }

        if(!supportedExtensions.containsAll(this.registry.getRequiredInstanceExtensions())) {
            throw new RuntimeException("Required instance extension not found");
        }

        Set<String> enabledExtensions = new HashSet<>();
        if(this.enableDebugUtils) {
            if(!supportedExtensions.contains(VK_EXT_DEBUG_UTILS_EXTENSION_NAME)) {
                throw new RuntimeException("Debug was enabled but EXTDebugUtils extension is not supported");
            }

            enabledExtensions.add(VK_EXT_DEBUG_UTILS_EXTENSION_NAME);
        }

        enabledExtensions.addAll(this.registry.getRequiredInstanceExtensions());
        enabledExtensions.addAll(this.registry.getOptionalInstanceExtensions().stream().filter(supportedExtensions::contains).toList());

        PointerBuffer extensionNames = stack.mallocPointer(enabledExtensions.size());
        for(String extension : enabledExtensions) {
            extensionNames.put(stack.UTF8(extension));
        }

        return extensionNames.rewind();
    }

    private long createDebugUtilsCallback(long pNext, MemoryStack stack) {
        if(!this.enableDebugUtils) {
            return pNext;
        }

        VkDebugUtilsMessengerCreateInfoEXT debugCreateInfo = VkDebugUtilsMessengerCreateInfoEXT.callocStack(stack);
        debugCreateInfo.sType(VK_STRUCTURE_TYPE_DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT);
        debugCreateInfo.pNext(pNext);
        debugCreateInfo.messageSeverity(MessageSeverity.allBits());
        debugCreateInfo.messageType(MessageType.allBits());
        debugCreateInfo.pfnUserCallback(this.debugUtilsCallback::vulkanCallbackFunction);

        return debugCreateInfo.address();
    }
}
