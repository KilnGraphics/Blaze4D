package me.hydos.rosella.vkobjects;

import me.hydos.rosella.logging.DebugLogger;
import org.lwjgl.PointerBuffer;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.vulkan.*;

import java.nio.IntBuffer;
import java.nio.LongBuffer;
import java.util.List;
import java.util.OptionalLong;

import static me.hydos.rosella.memory.Memory.asPtrBuffer;
import static me.hydos.rosella.util.VkUtils.ok;
import static org.lwjgl.vulkan.EXTDebugUtils.*;
import static org.lwjgl.vulkan.VK10.*;
import static org.lwjgl.vulkan.VK12.VK_API_VERSION_1_2;

/**
 * {@link me.hydos.rosella.Rosella} representation of a {@link VkInstance}. Contains a couple of useful things here and there and does most of the work for you.
 */
public class VulkanInstance {

    public final DebugLogger debugLogger;
    public final VkInstance rawInstance;
    public final OptionalLong messenger;

    public VulkanInstance(List<String> requestedValidationLayers, List<String> requestedExtensions, String applicationName, DebugLogger debugLogger) {
        this.debugLogger = debugLogger;

        boolean validationLayers = !requestedValidationLayers.isEmpty();

        try (MemoryStack stack = MemoryStack.stackPush()) {
            VkApplicationInfo applicationInfo = VkApplicationInfo.callocStack(stack)
                    .sType(VK_STRUCTURE_TYPE_APPLICATION_INFO)
                    .apiVersion(VK_API_VERSION_1_2)
                    .pEngineName(stack.UTF8Safe("Rosella"))
                    .engineVersion(VK_MAKE_VERSION(2, 0, 0))

                    .pApplicationName(stack.UTF8Safe(applicationName))
                    .applicationVersion(VK_MAKE_VERSION(1, 0, 0));

            VkInstanceCreateInfo createInfo = VkInstanceCreateInfo.callocStack(stack)
                    .sType(VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO)
                    .pApplicationInfo(applicationInfo)
                    .ppEnabledExtensionNames(getRequiredExtensions(requestedValidationLayers.size() != 0, requestedExtensions, stack));

            VkDebugUtilsMessengerCreateInfoEXT debugMessengerCreateInfo;

            {
                IntBuffer validationFeatures = stack.callocInt(6)
                        .put(EXTValidationFeatures.VK_VALIDATION_FEATURE_ENABLE_BEST_PRACTICES_EXT)
                        .put(EXTValidationFeatures.VK_VALIDATION_FEATURE_ENABLE_GPU_ASSISTED_EXT)
                        .put(EXTValidationFeatures.VK_VALIDATION_FEATURE_ENABLE_DEBUG_PRINTF_EXT)
                        .put(EXTValidationFeatures.VK_VALIDATION_FEATURE_ENABLE_SYNCHRONIZATION_VALIDATION_EXT)
                        .put(EXTValidationFeatures.VK_VALIDATION_FEATURE_ENABLE_GPU_ASSISTED_RESERVE_BINDING_SLOT_EXT)
                        .put(301);

                VkValidationFeaturesEXT extValidationFeatures = VkValidationFeaturesEXT.callocStack(stack)
                        .sType(EXTValidationFeatures.VK_STRUCTURE_TYPE_VALIDATION_FEATURES_EXT)
                        .pEnabledValidationFeatures(validationFeatures);

                debugMessengerCreateInfo = VkDebugUtilsMessengerCreateInfoEXT.callocStack(stack)
                        .sType(VK_STRUCTURE_TYPE_DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT)
                        .messageSeverity(VK_DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT | VK_DEBUG_UTILS_MESSAGE_SEVERITY_ERROR_BIT_EXT)
                        .messageType(VK_DEBUG_UTILS_MESSAGE_TYPE_GENERAL_BIT_EXT | VK_DEBUG_UTILS_MESSAGE_TYPE_VALIDATION_BIT_EXT | VK_DEBUG_UTILS_MESSAGE_TYPE_PERFORMANCE_BIT_EXT)
                        .pfnUserCallback(this::debugCallback)
                        .pNext(extValidationFeatures.address());
            }

            if (validationLayers) {
                // This is only while the instance is being created
                createInfo
                        .ppEnabledLayerNames(asPtrBuffer(requestedValidationLayers, stack))
                        .pNext(debugMessengerCreateInfo.address());
            }

            PointerBuffer pVkInstance = stack.mallocPointer(1);
            ok(vkCreateInstance(createInfo, null, pVkInstance));
            rawInstance = new VkInstance(pVkInstance.get(0), createInfo);

            if (validationLayers) {
                // This is after the instance has been created
                LongBuffer messenger = MemoryStack.stackLongs(0);
                ok(vkCreateDebugUtilsMessengerEXT(rawInstance, debugMessengerCreateInfo, null, messenger));
                this.messenger = OptionalLong.of(messenger.get());
            } else {
                this.messenger = OptionalLong.empty();
            }
        }
    }

    /**
     * Converts a {@link List} into a {@link PointerBuffer}
     *
     * @param list  the list to put into a {@link PointerBuffer}
     * @param stack the current {@link MemoryStack}
     * @return a valid {@link PointerBuffer}
     */
    public static PointerBuffer asPointerBuffer(List<String> list, MemoryStack stack) {
        PointerBuffer pBuffer = stack.mallocPointer(list.size());
        for (String object : list) {
            pBuffer.put(stack.UTF8Safe(object));
        }
        return pBuffer.rewind();
    }

    /**
     * Gets the required Extensions needed depending on what is being used
     *
     * @param useValidation       if true, an extra validation lay
     * @param requestedExtensions extensions requested by {@link me.hydos.rosella.Rosella} based on user choice and necessity
     * @param stack               the in use {@link MemoryStack}
     * @return a {@link PointerBuffer} created from the {@link List}
     */
    private PointerBuffer getRequiredExtensions(boolean useValidation, List<String> requestedExtensions, MemoryStack stack) {
        PointerBuffer extensions = stack.mallocPointer(requestedExtensions.size() + (useValidation ? 1 : 0));
        for (String requestedExtension : requestedExtensions) {
            extensions.put(stack.UTF8Safe(requestedExtension));
        }
        if (useValidation) {
            extensions.put(stack.UTF8(VK_EXT_DEBUG_UTILS_EXTENSION_NAME));
        }
        return extensions.rewind();
    }

    /**
     * Called when Vulkan decides to give us some information
     *
     * @param severity      the severity of the information
     * @param messageType   the type of information
     * @param pCallbackData im not sure about this
     * @param pUserData     im also not sure if this matters to us
     * @return if the Driver needs to throw VK_DEVICE_LOST or similar on the next Vulkan call, VK_TRUE will be returned.
     */
    private int debugCallback(int severity, int messageType, long pCallbackData, long pUserData) {
        VkDebugUtilsMessengerCallbackDataEXT callbackData = VkDebugUtilsMessengerCallbackDataEXT.create(pCallbackData);
        String message = callbackData.pMessageString();

        String msgSeverity = switch (severity) {
            case VK_DEBUG_UTILS_MESSAGE_SEVERITY_VERBOSE_BIT_EXT -> "VERBOSE";
            case VK_DEBUG_UTILS_MESSAGE_SEVERITY_INFO_BIT_EXT -> "INFO";
            case VK_DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT -> "WARNING";
            case VK_DEBUG_UTILS_MESSAGE_SEVERITY_ERROR_BIT_EXT -> "ERROR";
            default -> throw new IllegalStateException("Unexpected severity: " + severity);
        };

        return switch (messageType) {
            case VK_DEBUG_UTILS_MESSAGE_TYPE_GENERAL_BIT_EXT -> debugLogger.logGeneral(message, msgSeverity);
            case VK_DEBUG_UTILS_MESSAGE_TYPE_VALIDATION_BIT_EXT -> debugLogger.logValidation(message, msgSeverity);
            case VK_DEBUG_UTILS_MESSAGE_TYPE_PERFORMANCE_BIT_EXT -> debugLogger.logPerformance(message, msgSeverity);
            default -> debugLogger.logUnknown(message, msgSeverity);
        };
    }
}
