package me.hydos.rosella.device;

import me.hydos.rosella.device.init.DeviceBuilder;
import me.hydos.rosella.device.init.InitializationRegistry;
import me.hydos.rosella.device.init.VulkanInstance;
import me.hydos.rosella.device.init.features.RosellaLegacy;
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
@Deprecated
public class LegacyVulkanDevice {

    private static final Set<String> REQUIRED_EXTENSIONS = Collections.singleton(VK_KHR_SWAPCHAIN_EXTENSION_NAME);

    public final VulkanDevice newDevice;

    public final QueueFamilyIndices indices;
    public VkDevice rawDevice;
    public VkPhysicalDevice physicalDevice;

    public LegacyVulkanDevice(VulkanDevice device) {
        this.newDevice = device;
        this.indices = RosellaLegacy.getMetadata(device).indices();
        this.rawDevice = device.getDevice();
        this.physicalDevice = device.getDevice().getPhysicalDevice();
    }

    public LegacyVulkanDevice(VulkanInstance instance, InitializationRegistry registry) {
        DeviceBuilder builder = new DeviceBuilder(instance, registry);
        this.newDevice = builder.build();

        this.indices = RosellaLegacy.getMetadata(this.newDevice).indices();
        this.rawDevice = this.newDevice.getDevice();
        this.physicalDevice = this.rawDevice.getPhysicalDevice();
    }

    /**
     * @param common           the vulkan common variables
     * @param validationLayers the validation layers to use
     * @deprecated Use the other method to allow for more than just anisotropy
     */
    public LegacyVulkanDevice(VkCommon common, List<String> validationLayers) {
        this(common, validationLayers, deviceFeatures -> deviceFeatures
                .samplerAnisotropy(true)
                .depthClamp(true)
                .depthBounds(true));
    }

    public LegacyVulkanDevice(VkCommon common, List<String> validationLayers, Consumer<VkPhysicalDeviceFeatures> deviceFeatureCallback) {
        this.newDevice = null;

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
                Set<String> supportedExtensions = getExtensionStrings(device);

                if (deviceSuitable(device, common, supportedExtensions)) {
                    // FIXME this doesn't work if there are multiple gpus in the system, ex integrated and pcie
                    this.physicalDevice = device;
                    StringBuilder stringBuilder = new StringBuilder();
                    for (String extension : supportedExtensions) {
                        stringBuilder.append(extension).append(" ");
                    }
                    stringBuilder.deleteCharAt(stringBuilder.length() - 1); // remove extra space
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
                    .ppEnabledExtensionNames(asPtrBuffer(REQUIRED_EXTENSIONS));

            if (validationLayers.size() != 0) {
                deviceCreateInfo.ppEnabledLayerNames(asPtrBuffer(validationLayers));
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
    private boolean deviceSuitable(VkPhysicalDevice device, VkCommon common, Set<String> supportedExtensions) {
        QueueFamilyIndices indices = VkUtils.findQueueFamilies(device, common.surface);

        if (supportedExtensions.containsAll(REQUIRED_EXTENSIONS)) {
            try (MemoryStack stack = MemoryStack.stackPush()) {
                boolean swapChainAdequate;
                boolean anisotropySupported;

                SwapchainSupportDetails swapchainSupport = Swapchain.Companion.querySwapchainSupport(device, stack, common.surface); // TODO: unkotlinify the swapchain
                swapChainAdequate = swapchainSupport.formats.hasRemaining() && swapchainSupport.presentModes.hasRemaining(); // Check if the swapchain has valid formats and present modes available
                VkPhysicalDeviceFeatures supportedFeatures = VkPhysicalDeviceFeatures.callocStack(stack);
                vkGetPhysicalDeviceFeatures(device, supportedFeatures);
                //this.deviceFeatures = getFeatures(supportedFeatures);
                anisotropySupported = supportedFeatures.samplerAnisotropy();

                return indices.isComplete() && swapChainAdequate && anisotropySupported;
            }
        }
        return false;
    }

    /**
     * @param device the device to check
     * @return all of the supported extensions of the device as a set of strings.
     */
    private Set<String> getExtensionStrings(VkPhysicalDevice device) {
        try (MemoryStack stack = MemoryStack.stackPush()) {
            IntBuffer extensionCount = stack.ints(0);
            ok(vkEnumerateDeviceExtensionProperties(device, (CharSequence) null, extensionCount, null));
            VkExtensionProperties.Buffer availableExtensions = VkExtensionProperties.callocStack(extensionCount.get(0), stack);
            ok(vkEnumerateDeviceExtensionProperties(device, (CharSequence) null, extensionCount, availableExtensions));
            return availableExtensions.stream()
                    .map(VkExtensionProperties::extensionNameString)
                    .collect(Collectors.toSet());
        }
    }
}
