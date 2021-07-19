package me.hydos.rosella.device;

import me.hydos.rosella.render.swapchain.Swapchain;
import me.hydos.rosella.render.swapchain.SwapchainSupportDetails;
import me.hydos.rosella.util.VkUtils;
import me.hydos.rosella.vkobjects.VkCommon;
import org.lwjgl.PointerBuffer;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.vulkan.*;

import java.nio.IntBuffer;
import java.util.Collections;
import java.util.List;
import java.util.Set;
import java.util.function.Consumer;
import java.util.stream.Collectors;

import static me.hydos.rosella.memory.Memory.asPtrBuffer;
import static me.hydos.rosella.util.VkUtils.ok;
import static org.lwjgl.vulkan.KHRSwapchain.VK_KHR_SWAPCHAIN_EXTENSION_NAME;
import static org.lwjgl.vulkan.VK10.*;

/**
 * The object which represents both a Physical and Logical device used by {@link me.hydos.rosella.Rosella}
 */
public class VulkanDevice {

    private static final Set<String> REQUIRED_EXTENSIONS = Collections.singleton(VK_KHR_SWAPCHAIN_EXTENSION_NAME);

    public final QueueFamilyIndices indices;
    public VkDevice rawDevice;
    public VkPhysicalDevice physicalDevice;
    public DeviceFeatures deviceFeatures;
    public Properties properties;

    /**
     * @param common           the vulkan common variables
     * @param validationLayers the validation layers to use
     * @deprecated Use the other method to allow for more than just anisotropy
     */
    @Deprecated
    public VulkanDevice(VkCommon common, List<String> validationLayers) {
        this(common, validationLayers, deviceFeatures -> deviceFeatures
                .samplerAnisotropy(true)
                .depthClamp(true)
                .depthBounds(true));
    }

    public VulkanDevice(VkCommon common, List<String> validationLayers, Consumer<VkPhysicalDeviceFeatures> deviceFeatureCallback) {
        try (MemoryStack stack = MemoryStack.stackPush()) {
            IntBuffer pPhysicalDeviceCount = stack.ints(0);
            ok(vkEnumeratePhysicalDevices(common.vkInstance.rawInstance, pPhysicalDeviceCount, null));
            // Unless the user is somehow hot swapping gpu's while the engine is running, this is 100% safe to do.
            if (pPhysicalDeviceCount.get(0) == 0) {
                throw new RuntimeException("Your system does not have vulkan support. Make sure your drivers are up to date.");
            }

            // Retrieve the VkPhysicalDevice
            PointerBuffer pPhysicalDevices = stack.mallocPointer(pPhysicalDeviceCount.get(0));
            ok(vkEnumeratePhysicalDevices(common.vkInstance.rawInstance, pPhysicalDeviceCount, pPhysicalDevices));

            for (int i = 0; i < pPhysicalDeviceCount.capacity(); i++) {
                VkPhysicalDevice device = new VkPhysicalDevice(pPhysicalDevices.get(i), common.vkInstance.rawInstance);

                if (deviceSuitable(device, common)) {
                    this.physicalDevice = device;
                    setDeviceInfo();
                    break;
                }
            }

            // Create a VkLogicalDevice
            this.indices = VkUtils.findQueueFamilies(physicalDevice, common.surface);
            int[] uniqueQueueFamilies = indices.unique();
            VkDeviceQueueCreateInfo.Buffer queueCreateInfos = VkDeviceQueueCreateInfo.callocStack(uniqueQueueFamilies.length, stack);

            for (int i = 0; i < uniqueQueueFamilies.length; i++) {
                queueCreateInfos.get(i)
                        .sType(VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO)
                        .queueFamilyIndex(uniqueQueueFamilies[i])
                        .pQueuePriorities(stack.floats(1.0f));
            }

            VkPhysicalDeviceFeatures deviceFeatures = VkPhysicalDeviceFeatures.callocStack(stack);
            deviceFeatureCallback.accept(deviceFeatures);

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
    }

    private void setDeviceInfo() {
        try (MemoryStack stack = MemoryStack.stackPush()) {
            VkPhysicalDeviceProperties properties = VkPhysicalDeviceProperties.callocStack(stack);
            vkGetPhysicalDeviceProperties(physicalDevice, properties);

            this.properties = new Properties(properties);
        }
    }

    /**
     * Waits for the underlying device which this Vulkan instance is operating on to be idle
     */
    public void waitForIdle() {
        ok(vkDeviceWaitIdle(rawDevice));
    }

    /**
     * Checks if a device is suitable enough for our use.
     *
     * @param device the {@link VkPhysicalDevice} to test
     * @param common the Rosella shared fields
     * @return if the physical device can be used
     */
    private boolean deviceSuitable(VkPhysicalDevice device, VkCommon common) {
        QueueFamilyIndices indices = VkUtils.findQueueFamilies(device, common.surface);

        if (deviceSupportsExtensions(device)) {
            try (MemoryStack stack = MemoryStack.stackPush()) {
                boolean swapChainAdequate;
                boolean anisotropySupported;

                SwapchainSupportDetails swapchainSupport = Swapchain.Companion.querySwapchainSupport(device, stack, common.surface); // TODO: unkotlinify the swapchain
                swapChainAdequate = swapchainSupport.formats.hasRemaining() && swapchainSupport.presentModes.hasRemaining(); // Check if the swapchain has valid formats and present modes available
                VkPhysicalDeviceFeatures supportedFeatures = VkPhysicalDeviceFeatures.callocStack(stack);
                vkGetPhysicalDeviceFeatures(device, supportedFeatures);
                this.deviceFeatures = getFeatures(supportedFeatures);
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
    private DeviceFeatures getFeatures(VkPhysicalDeviceFeatures supportedFeatures) {
        return new DeviceFeatures(
                supportedFeatures.robustBufferAccess(),
                supportedFeatures.fullDrawIndexUint32(),
                supportedFeatures.imageCubeArray(),
                supportedFeatures.independentBlend(),
                supportedFeatures.geometryShader(),
                supportedFeatures.tessellationShader(),
                supportedFeatures.sampleRateShading(),
                supportedFeatures.dualSrcBlend(),
                supportedFeatures.logicOp(),
                supportedFeatures.multiDrawIndirect(),
                supportedFeatures.drawIndirectFirstInstance(),
                supportedFeatures.depthClamp(),
                supportedFeatures.depthBiasClamp(),
                supportedFeatures.fillModeNonSolid(),
                supportedFeatures.depthBounds(),
                supportedFeatures.wideLines(),
                supportedFeatures.largePoints(),
                supportedFeatures.alphaToOne(),
                supportedFeatures.multiViewport(),
                supportedFeatures.samplerAnisotropy(),
                supportedFeatures.textureCompressionETC2(),
                supportedFeatures.textureCompressionASTC_LDR(),
                supportedFeatures.textureCompressionBC(),
                supportedFeatures.occlusionQueryPrecise(),
                supportedFeatures.pipelineStatisticsQuery(),
                supportedFeatures.vertexPipelineStoresAndAtomics(),
                supportedFeatures.fragmentStoresAndAtomics(),
                supportedFeatures.shaderTessellationAndGeometryPointSize(),
                supportedFeatures.shaderImageGatherExtended(),
                supportedFeatures.shaderStorageImageExtendedFormats(),
                supportedFeatures.shaderStorageImageMultisample(),
                supportedFeatures.shaderStorageImageReadWithoutFormat(),
                supportedFeatures.shaderStorageImageWriteWithoutFormat(),
                supportedFeatures.shaderUniformBufferArrayDynamicIndexing(),
                supportedFeatures.shaderSampledImageArrayDynamicIndexing(),
                supportedFeatures.shaderStorageBufferArrayDynamicIndexing(),
                supportedFeatures.shaderStorageImageArrayDynamicIndexing(),
                supportedFeatures.shaderClipDistance(),
                supportedFeatures.shaderCullDistance(),
                supportedFeatures.shaderFloat64(),
                supportedFeatures.shaderInt64(),
                supportedFeatures.shaderInt16(),
                supportedFeatures.shaderResourceResidency(),
                supportedFeatures.shaderResourceMinLod(),
                supportedFeatures.sparseBinding(),
                supportedFeatures.sparseResidencyBuffer(),
                supportedFeatures.sparseResidencyImage2D(),
                supportedFeatures.sparseResidencyImage3D(),
                supportedFeatures.sparseResidency2Samples(),
                supportedFeatures.sparseResidency4Samples(),
                supportedFeatures.sparseResidency8Samples(),
                supportedFeatures.sparseResidency16Samples(),
                supportedFeatures.sparseResidencyAliased(),
                supportedFeatures.variableMultisampleRate(),
                supportedFeatures.inheritedQueries());
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
            ok(vkEnumerateDeviceExtensionProperties(device, (CharSequence) null, extensionCount, availableExtensions));
            Set<String> collect = availableExtensions.stream()
                    .map(VkExtensionProperties::extensionNameString)
                    .collect(Collectors.toSet());
            return collect.containsAll(REQUIRED_EXTENSIONS);
        }
    }

    public record DeviceFeatures(boolean robustBufferAccess, boolean fullDrawIndexUint32, boolean imageCubeArray,
                                 boolean independentBlend, boolean geometryShader, boolean tessellationShader,
                                 boolean sampleRateShading, boolean dualSrcBlend, boolean logicOp,
                                 boolean multiDrawIndirect, boolean drawIndirectFirstInstance, boolean depthClamp,
                                 boolean depthBiasClamp, boolean fillModeNonSolid, boolean depthBounds,
                                 boolean wideLines,
                                 boolean largePoints, boolean alphaToOne, boolean multiViewport,
                                 boolean samplerAnisotropy,
                                 boolean textureCompressionETC2, boolean textureCompressionASTC_LDR,
                                 boolean textureCompressionBC, boolean occlusionQueryPrecise,
                                 boolean pipelineStatisticsQuery, boolean vertexPipelineStoresAndAtomics,
                                 boolean fragmentStoresAndAtomics, boolean shaderTessellationAndGeometryPointSize,
                                 boolean shaderImageGatherExtended, boolean shaderStorageImageExtendedFormats,
                                 boolean shaderStorageImageMultisample, boolean shaderStorageImageReadWithoutFormat,
                                 boolean shaderStorageImageWriteWithoutFormat,
                                 boolean shaderUniformBufferArrayDynamicIndexing,
                                 boolean shaderSampledImageArrayDynamicIndexing,
                                 boolean shaderStorageBufferArrayDynamicIndexing,
                                 boolean shaderStorageImageArrayDynamicIndexing, boolean shaderClipDistance,
                                 boolean shaderCullDistance, boolean shaderFloat64, boolean shaderInt64,
                                 boolean shaderInt16, boolean shaderResourceResidency, boolean shaderResourceMinLod,
                                 boolean sparseBinding, boolean sparseResidencyBuffer, boolean sparseResidencyImage2D,
                                 boolean sparseResidencyImage3D, boolean sparseResidency2Samples,
                                 boolean sparseResidency4Samples, boolean sparseResidency8Samples,
                                 boolean sparseResidency16Samples, boolean sparseResidencyAliased,
                                 boolean variableMultisampleRate, boolean inheritedQueries) {
    }

    public static class Properties {

        public final int deviceId;
        public final String deviceName;
        public final String apiVersion;
        public final int driverVersion;
        public final int vendorId;

        public Properties(VkPhysicalDeviceProperties properties) {
            this.deviceId = properties.deviceID();
            this.deviceName = properties.deviceNameString();
            this.apiVersion = fromVkVersion(properties.apiVersion());
            this.driverVersion = properties.driverVersion();
            this.vendorId = properties.vendorID();
        }

        /**
         * Turns integer VkVersion into a string version
         *
         * @param apiVersion the integer passed from vulkan
         * @return a readable string
         */
        private String fromVkVersion(int apiVersion) {
            int major = VK_VERSION_MAJOR(apiVersion);
            int minor = VK_VERSION_MINOR(apiVersion);
            int patch = VK_VERSION_PATCH(apiVersion);
            return major + "." + minor + "." + patch;
        }
    }
}
