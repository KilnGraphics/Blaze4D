package me.hydos.rosella.device;

import me.hydos.rosella.render.swapchain.Swapchain;
import me.hydos.rosella.render.swapchain.SwapchainSupportDetails;
import me.hydos.rosella.vkobjects.VkCommon;
import org.lwjgl.PointerBuffer;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.vulkan.*;

import java.nio.IntBuffer;
import java.util.Collections;
import java.util.List;
import java.util.Set;
import java.util.stream.Collectors;

import static me.hydos.rosella.render.VkKt.findQueueFamilies;
import static me.hydos.rosella.render.util.VkUtilsKt.ok;
import static me.hydos.rosella.util.Memory.asPtrBuffer;
import static org.lwjgl.vulkan.KHRSwapchain.VK_KHR_SWAPCHAIN_EXTENSION_NAME;
import static org.lwjgl.vulkan.VK10.*;

/**
 * The object which represents both a Physical and Logical device used by {@link me.hydos.rosella.Rosella}
 */
public class VulkanDevice {

    private static final Set<String> REQUIRED_EXTENSIONS = Collections.singleton(VK_KHR_SWAPCHAIN_EXTENSION_NAME);
    private static IntBuffer pPhysicalDeviceCount;
    public final QueueFamilyIndices indices;
    public VkDevice rawDevice;
    public VkPhysicalDevice physicalDevice;
    public PhysicalDeviceFeatures physicalDeviceFeatures;

    public VulkanDevice(VkCommon common, List<String> validationLayers) {
        if (systemSupportsVulkan(common)) {
            try (MemoryStack stack = MemoryStack.stackPush()) {
                // Retrieve the VkPhysicalDevice
                PointerBuffer pPhysicalDevices = stack.mallocPointer(pPhysicalDeviceCount.get(0));
                ok(vkEnumeratePhysicalDevices(common.vkInstance.rawInstance, pPhysicalDeviceCount, pPhysicalDevices));

                for (int i = 0; i < pPhysicalDeviceCount.capacity(); i++) {
                    VkPhysicalDevice device = new VkPhysicalDevice(pPhysicalDevices.get(i), common.vkInstance.rawInstance);

                    if (deviceSuitable(device, common)) {
                        this.physicalDevice = device;
                        break;
                    }
                }

                // Create a VkLogicalDevice
                this.indices = findQueueFamilies(physicalDevice, common.surface);
                int[] uniqueQueueFamilies = indices.unique();
                VkDeviceQueueCreateInfo.Buffer queueCreateInfos = VkDeviceQueueCreateInfo.callocStack(uniqueQueueFamilies.length, stack);

                for (int i = 0; i < uniqueQueueFamilies.length; i++) {
                    queueCreateInfos.get(i)
                            .sType(VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO)
                            .queueFamilyIndex(uniqueQueueFamilies[i])
                            .pQueuePriorities(stack.floats(1.0f));
                }

                VkPhysicalDeviceFeatures deviceFeatures = VkPhysicalDeviceFeatures.callocStack(stack)
                        .samplerAnisotropy(true);
                VkDeviceCreateInfo deviceCreateInfo = VkDeviceCreateInfo.callocStack(stack)
                        .sType(VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO)
                        .pQueueCreateInfos(queueCreateInfos)
                        .pEnabledFeatures(deviceFeatures)
                        .ppEnabledExtensionNames(asPtrBuffer(REQUIRED_EXTENSIONS, stack));

                if (validationLayers.size() != 0) {
                    deviceCreateInfo.ppEnabledLayerNames(asPtrBuffer(validationLayers, stack));
                }

                PointerBuffer pDevice = stack.pointers(VK_NULL_HANDLE);
                ok(vkCreateDevice(physicalDevice, deviceCreateInfo, null, pDevice));
                rawDevice = new VkDevice(pDevice.get(0), physicalDevice, deviceCreateInfo);
            }
        } else {
            throw new RuntimeException("Your system does not have vulkan support. Make sure your drivers are up to date.");
        }
    }

    /**
     * Checks if a device is suitable enough for our use.
     *
     * @param device the {@link VkPhysicalDevice} to test
     * @param common the Rosella shared fields
     * @return if the physical device can be used
     */
    private boolean deviceSuitable(VkPhysicalDevice device, VkCommon common) {
        QueueFamilyIndices indices = findQueueFamilies(device, common.surface);

        if (deviceSupportsExtensions(device)) {
            try (MemoryStack stack = MemoryStack.stackPush()) {
                boolean swapChainAdequate;
                boolean anisotropySupported;

                SwapchainSupportDetails swapchainSupport = Swapchain.Companion.querySwapchainSupport(device, stack, common.surface); // TODO: unkotlinify the swapchain
                swapChainAdequate = swapchainSupport.formats.hasRemaining() && swapchainSupport.presentModes.hasRemaining(); // Check if the swapchain has valid formats and present modes available
                VkPhysicalDeviceFeatures supportedFeatures = VkPhysicalDeviceFeatures.mallocStack(stack);
                vkGetPhysicalDeviceFeatures(device, supportedFeatures);
                this.physicalDeviceFeatures = getFeatures(supportedFeatures);
                anisotropySupported = supportedFeatures.samplerAnisotropy();

                return indices.isComplete() && swapChainAdequate && anisotropySupported;
            }
        }
        return false;
    }

    /**
     * Copies all of the features out of {@link VkPhysicalDeviceFeatures} so it can be read later on when the stack is freed.
     *
     * @param supportedFeatures the Vulkan struct
     * @return a usable class
     */
    private PhysicalDeviceFeatures getFeatures(VkPhysicalDeviceFeatures supportedFeatures) {
        PhysicalDeviceFeatures features = new PhysicalDeviceFeatures();
        features.robustBufferAccess = supportedFeatures.robustBufferAccess();
        features.fullDrawIndexUint32 = supportedFeatures.fullDrawIndexUint32();
        features.imageCubeArray = supportedFeatures.imageCubeArray();
        features.independentBlend = supportedFeatures.independentBlend();
        features.geometryShader = supportedFeatures.geometryShader();
        features.tessellationShader = supportedFeatures.tessellationShader();
        features.sampleRateShading = supportedFeatures.sampleRateShading();
        features.dualSrcBlend = supportedFeatures.dualSrcBlend();
        features.logicOp = supportedFeatures.logicOp();
        features.multiDrawIndirect = supportedFeatures.multiDrawIndirect();
        features.drawIndirectFirstInstance = supportedFeatures.drawIndirectFirstInstance();
        features.depthClamp = supportedFeatures.depthClamp();
        features.depthBiasClamp = supportedFeatures.depthBiasClamp();
        features.fillModeNonSolid = supportedFeatures.fillModeNonSolid();
        features.depthBounds = supportedFeatures.depthBounds();
        features.wideLines = supportedFeatures.wideLines();
        features.largePoints = supportedFeatures.largePoints();
        features.alphaToOne = supportedFeatures.alphaToOne();
        features.multiViewport = supportedFeatures.multiViewport();
        features.samplerAnisotropy = supportedFeatures.samplerAnisotropy();
        features.textureCompressionETC2 = supportedFeatures.textureCompressionETC2();
        features.textureCompressionASTC_LDR = supportedFeatures.textureCompressionASTC_LDR();
        features.textureCompressionBC = supportedFeatures.textureCompressionBC();
        features.occlusionQueryPrecise = supportedFeatures.occlusionQueryPrecise();
        features.pipelineStatisticsQuery = supportedFeatures.pipelineStatisticsQuery();
        features.vertexPipelineStoresAndAtomics = supportedFeatures.vertexPipelineStoresAndAtomics();
        features.fragmentStoresAndAtomics = supportedFeatures.fragmentStoresAndAtomics();
        features.shaderTessellationAndGeometryPointSize = supportedFeatures.shaderTessellationAndGeometryPointSize();
        features.shaderImageGatherExtended = supportedFeatures.shaderImageGatherExtended();
        features.shaderStorageImageExtendedFormats = supportedFeatures.shaderStorageImageExtendedFormats();
        features.shaderStorageImageMultisample = supportedFeatures.shaderStorageImageMultisample();
        features.shaderStorageImageReadWithoutFormat = supportedFeatures.shaderStorageImageReadWithoutFormat();
        features.shaderStorageImageWriteWithoutFormat = supportedFeatures.shaderStorageImageWriteWithoutFormat();
        features.shaderUniformBufferArrayDynamicIndexing = supportedFeatures.shaderUniformBufferArrayDynamicIndexing();
        features.shaderSampledImageArrayDynamicIndexing = supportedFeatures.shaderSampledImageArrayDynamicIndexing();
        features.shaderStorageBufferArrayDynamicIndexing = supportedFeatures.shaderStorageBufferArrayDynamicIndexing();
        features.shaderStorageImageArrayDynamicIndexing = supportedFeatures.shaderStorageImageArrayDynamicIndexing();
        features.shaderClipDistance = supportedFeatures.shaderClipDistance();
        features.shaderCullDistance = supportedFeatures.shaderCullDistance();
        features.shaderFloat64 = supportedFeatures.shaderFloat64();
        features.shaderInt64 = supportedFeatures.shaderInt64();
        features.shaderInt16 = supportedFeatures.shaderInt16();
        features.shaderResourceResidency = supportedFeatures.shaderResourceResidency();
        features.shaderResourceMinLod = supportedFeatures.shaderResourceMinLod();
        features.sparseBinding = supportedFeatures.sparseBinding();
        features.sparseResidencyBuffer = supportedFeatures.sparseResidencyBuffer();
        features.sparseResidencyImage2D = supportedFeatures.sparseResidencyImage2D();
        features.sparseResidencyImage3D = supportedFeatures.sparseResidencyImage3D();
        features.sparseResidency2Samples = supportedFeatures.sparseResidency2Samples();
        features.sparseResidency4Samples = supportedFeatures.sparseResidency4Samples();
        features.sparseResidency8Samples = supportedFeatures.sparseResidency8Samples();
        features.sparseResidency16Samples = supportedFeatures.sparseResidency16Samples();
        features.sparseResidencyAliased = supportedFeatures.sparseResidencyAliased();
        features.variableMultisampleRate = supportedFeatures.variableMultisampleRate();
        features.inheritedQueries = supportedFeatures.inheritedQueries();
        return features;
    }

    /**
     * Checks if a device supports the required extensions.
     *
     * @param device the device to check
     * @return if the device supports all required extensions
     */
    private boolean deviceSupportsExtensions(VkPhysicalDevice device) {
        try (MemoryStack stack = MemoryStack.stackPush()) {
            IntBuffer extensionCount = stack.ints(0);
            ok(vkEnumerateDeviceExtensionProperties(device, (CharSequence) null, extensionCount, null));
            VkExtensionProperties.Buffer availableExtensions = VkExtensionProperties.callocStack(extensionCount.get(0), stack);
            return availableExtensions.stream()
                    .map(VkExtensionProperties::extensionNameString)
                    .collect(Collectors.toSet())
                    .containsAll(REQUIRED_EXTENSIONS);
        }
    }

    /**
     * Checks if the system has any GPU's which can be used with vulkan
     *
     * @param common the common constants
     * @return if the system supports Vulkan
     */
    private boolean systemSupportsVulkan(VkCommon common) {
        try (MemoryStack stack = MemoryStack.stackPush()) {
            IntBuffer pPhysicalDeviceCount = stack.ints(0);
            ok(vkEnumeratePhysicalDevices(common.vkInstance.rawInstance, pPhysicalDeviceCount, null));
            // Unless the user is somehow hot swapping gpu's while the engine is running, this is 100% safe to do.
            VulkanDevice.pPhysicalDeviceCount = pPhysicalDeviceCount;
            return pPhysicalDeviceCount.get(0) != 0;
        }
    }

    public static class PhysicalDeviceFeatures {
        public boolean robustBufferAccess;
        public boolean fullDrawIndexUint32;
        public boolean imageCubeArray;
        public boolean independentBlend;
        public boolean geometryShader;
        public boolean tessellationShader;
        public boolean sampleRateShading;
        public boolean dualSrcBlend;
        public boolean logicOp;
        public boolean multiDrawIndirect;
        public boolean drawIndirectFirstInstance;
        public boolean depthClamp;
        public boolean depthBiasClamp;
        public boolean fillModeNonSolid;
        public boolean depthBounds;
        public boolean wideLines;
        public boolean largePoints;
        public boolean alphaToOne;
        public boolean multiViewport;
        public boolean samplerAnisotropy;
        public boolean textureCompressionETC2;
        public boolean textureCompressionASTC_LDR;
        public boolean textureCompressionBC;
        public boolean occlusionQueryPrecise;
        public boolean pipelineStatisticsQuery;
        public boolean vertexPipelineStoresAndAtomics;
        public boolean fragmentStoresAndAtomics;
        public boolean shaderTessellationAndGeometryPointSize;
        public boolean shaderImageGatherExtended;
        public boolean shaderStorageImageExtendedFormats;
        public boolean shaderStorageImageMultisample;
        public boolean shaderStorageImageReadWithoutFormat;
        public boolean shaderStorageImageWriteWithoutFormat;
        public boolean shaderUniformBufferArrayDynamicIndexing;
        public boolean shaderSampledImageArrayDynamicIndexing;
        public boolean shaderStorageBufferArrayDynamicIndexing;
        public boolean shaderStorageImageArrayDynamicIndexing;
        public boolean shaderClipDistance;
        public boolean shaderCullDistance;
        public boolean shaderFloat64;
        public boolean shaderInt64;
        public boolean shaderInt16;
        public boolean shaderResourceResidency;
        public boolean shaderResourceMinLod;
        public boolean sparseBinding;
        public boolean sparseResidencyBuffer;
        public boolean sparseResidencyImage2D;
        public boolean sparseResidencyImage3D;
        public boolean sparseResidency2Samples;
        public boolean sparseResidency4Samples;
        public boolean sparseResidency8Samples;
        public boolean sparseResidency16Samples;
        public boolean sparseResidencyAliased;
        public boolean variableMultisampleRate;
        public boolean inheritedQueries;
    }
}
