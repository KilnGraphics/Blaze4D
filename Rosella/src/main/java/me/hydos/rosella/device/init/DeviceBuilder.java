package me.hydos.rosella.device.init;

import me.hydos.rosella.Rosella;
import me.hydos.rosella.device.VulkanDevice;
import me.hydos.rosella.device.VulkanQueue;
import me.hydos.rosella.device.init.features.ApplicationFeature;
import me.hydos.rosella.util.NamedID;
import me.hydos.rosella.util.VkUtils;
import org.jetbrains.annotations.NotNull;
import org.lwjgl.PointerBuffer;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.vulkan.*;

import java.nio.FloatBuffer;
import java.nio.IntBuffer;
import java.util.*;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.Future;

/**
 * Used to build devices.
 *
 * This class will select and create the best device based on the criteria specified by the provided InitializationRegistry.
 */
public class DeviceBuilder {

    private final List<ApplicationFeature> applicationFeatures;
    private final Set<NamedID> requiredFeatures;
    private final VulkanInstance instance;

    public DeviceBuilder(@NotNull VulkanInstance instance, @NotNull InitializationRegistry registry) {
        this.instance = instance;
        this.applicationFeatures = registry.getOrderedFeatures();
        this.requiredFeatures = registry.getRequiredApplicationFeatures();
    }

    /**
     * Enumerates all available devices and selects the best. If no compatible device can be found throws a
     * runtime error.
     *
     * @return The initialized vulkan device
     * @throws RuntimeException if no compatible device can be found or an error occurs
     */
    public VulkanDevice build() throws RuntimeException {
        List<DeviceMeta> devices = new ArrayList<>();
        try (MemoryStack stack = MemoryStack.stackPush()) {
            IntBuffer deviceCount = stack.mallocInt(1);
            VkUtils.ok(VK10.vkEnumeratePhysicalDevices(this.instance.getInstance(), deviceCount, null));

            PointerBuffer pPhysicalDevices = stack.mallocPointer(deviceCount.get(0));
            VkUtils.ok(VK10.vkEnumeratePhysicalDevices(this.instance.getInstance(), deviceCount, pPhysicalDevices));

            for(int i = 0; i < deviceCount.get(0); i++) {
                devices.add(new DeviceMeta(new VkPhysicalDevice(pPhysicalDevices.get(i), this.instance.getInstance()), stack));
            }

            devices.forEach(DeviceMeta::processSupport);
            devices.sort((a, b) -> { // This is sorting in descending order so that we can use the first device
                if(!a.isValid() || !b.isValid()) {
                    if(a.isValid()) {
                        return -1;
                    }
                    if(b.isValid()) {
                        return 1;
                    }
                    return 0;
                }
                if(a.getFeatureRanking() != b.getFeatureRanking()) {
                    return (int) (b.getFeatureRanking() - a.getFeatureRanking());
                }
                if(a.getPerformanceRanking() != b.getPerformanceRanking()) {
                    return b.getPerformanceRanking() - a.getPerformanceRanking();
                }
                return 0;
            });

            DeviceMeta selectedDevice = devices.get(0);
            if(!selectedDevice.isValid()) {
                throw new RuntimeException("Failed to find suitable device");
            }

            return selectedDevice.createDevice();
        }
    }

    public class DeviceMeta implements DeviceBuildInformation, DeviceBuildConfigurator {
        private final MemoryStack stack;

        private final Set<NamedID> unsatisfiedRequirements = new HashSet<>();
        private final Map<NamedID, ApplicationFeature.Instance> features = new HashMap<>();
        private final ArrayList<ApplicationFeature.Instance> sortedFeatures = new ArrayList<>();

        private final VkPhysicalDevice physicalDevice;
        private final VkPhysicalDeviceProperties properties;
        private final VkPhysicalDeviceFeatures availableFeatures;
        private final Map<String, VkExtensionProperties> extensionProperties;
        private final List<VkQueueFamilyProperties> queueFamilyProperties;

        private boolean isBuilding = false;
        private final List<QueueRequest> queueRequests = new ArrayList<>();
        private final Set<String> enabledExtensions = new HashSet<>();
        private final VkPhysicalDeviceFeatures enabledFeatures;

        private DeviceMeta(VkPhysicalDevice physicalDevice, MemoryStack stack) {
            this.stack = stack;
            this.physicalDevice = physicalDevice;
            this.unsatisfiedRequirements.addAll(requiredFeatures);
            applicationFeatures.forEach(feature -> sortedFeatures.add(feature.createInstance()));
            this.sortedFeatures.forEach(feature -> features.put(feature.getFeatureName(), feature));

            IntBuffer count = stack.mallocInt(1);

            this.properties = VkPhysicalDeviceProperties.mallocStack(stack);
            VK10.vkGetPhysicalDeviceProperties(physicalDevice, this.properties);

            this.availableFeatures = VkPhysicalDeviceFeatures.mallocStack(stack);
            VK10.vkGetPhysicalDeviceFeatures(physicalDevice, availableFeatures);

            VK10.vkGetPhysicalDeviceQueueFamilyProperties(physicalDevice, count, null);
            VkQueueFamilyProperties.Buffer queueFamilyPropertiesBuffer = VkQueueFamilyProperties.mallocStack(count.get(0), stack);
            VK10.vkGetPhysicalDeviceQueueFamilyProperties(physicalDevice, count, queueFamilyPropertiesBuffer);
            ArrayList<VkQueueFamilyProperties> queueFamilyPropertiesList = new ArrayList<>();
            for(int i = 0; i < count.get(0); i++) {
                queueFamilyPropertiesList.add(queueFamilyPropertiesBuffer.get(i));
            }
            this.queueFamilyProperties = Collections.unmodifiableList(queueFamilyPropertiesList);

            VkUtils.ok(VK10.vkEnumerateDeviceExtensionProperties(this.physicalDevice, (CharSequence) null, count, null));
            VkExtensionProperties.Buffer extensionPropertiesBuffer = VkExtensionProperties.mallocStack(count.get(0), stack);
            VkUtils.ok(VK10.vkEnumerateDeviceExtensionProperties(this.physicalDevice, (CharSequence) null, count, extensionPropertiesBuffer));
            Map<String, VkExtensionProperties> extensionPropertiesMap = new HashMap<>();
            for(int i = 0; i < count.get(0); i++) {
                VkExtensionProperties properties = extensionPropertiesBuffer.get(i);
                extensionPropertiesMap.put(properties.extensionNameString(), properties);
            }
            this.extensionProperties = Collections.unmodifiableMap(extensionPropertiesMap);

            this.enabledFeatures = VkPhysicalDeviceFeatures.callocStack(stack);
        }

        private void processSupport() {
            for(ApplicationFeature.Instance feature : this.sortedFeatures) {
                try {
                    feature.testFeatureSupport(this);
                    if(feature.isSupported()) {
                        this.unsatisfiedRequirements.remove(feature.getFeatureName());
                    }
                } catch (Exception ex) {
                    Rosella.LOGGER.warn("Exception during support test for feature \"" + feature.getFeatureName() + "\"", ex);
                }
            }
        }

        /**
         * @return true if all required features are met by this device.
         */
        private boolean isValid() {
            return unsatisfiedRequirements.isEmpty();
        }

        /**
         * @return A ranking based on what features are supported. The greater the better.
         */
        private long getFeatureRanking() {
            return this.sortedFeatures.stream().filter(ApplicationFeature.Instance::isSupported).count();
        }

        /**
         * @return A ranking based on what the expected performance of the device is. The greater the better.
         */
        private int getPerformanceRanking() {
            return switch (properties.deviceType()) {
                case VK10.VK_PHYSICAL_DEVICE_TYPE_CPU, VK10.VK_PHYSICAL_DEVICE_TYPE_OTHER -> 0;
                case VK10.VK_PHYSICAL_DEVICE_TYPE_VIRTUAL_GPU -> 1;
                case VK10.VK_PHYSICAL_DEVICE_TYPE_INTEGRATED_GPU -> 2;
                case VK10.VK_PHYSICAL_DEVICE_TYPE_DISCRETE_GPU -> 3;
                default -> throw new RuntimeException("Device type was not recognized!");
            };
        }

        private VulkanDevice createDevice() {
            assert(!this.isBuilding);
            this.isBuilding = true;

            Map<NamedID, Object> enabledFeatures = new HashMap<>();

            for(ApplicationFeature.Instance feature : this.sortedFeatures) {
                try {
                    if(feature.isSupported()) {
                        Object metadata = feature.enableFeature(this);
                        enabledFeatures.put(feature.getFeatureName(), metadata);
                    }
                } catch (Exception ex) {
                    Rosella.LOGGER.warn("Exception while enabling feature \"" + feature.getFeatureName() + "\"", ex);
                }
            }

            VkDeviceCreateInfo deviceInfo = VkDeviceCreateInfo.callocStack(this.stack);
            deviceInfo.sType(VK10.VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO);
            deviceInfo.pQueueCreateInfos(this.generateQueueMappings());
            deviceInfo.ppEnabledExtensionNames(this.generateEnabledExtensionNames());
            deviceInfo.pEnabledFeatures(this.enabledFeatures);

            PointerBuffer pDevice = this.stack.mallocPointer(1);
            VkUtils.ok(VK10.vkCreateDevice(this.physicalDevice, deviceInfo, null, pDevice));

            VkDevice device = new VkDevice(pDevice.get(0), this.physicalDevice, deviceInfo);

            this.fulfillQueueRequests(device);

            return new VulkanDevice(device, enabledFeatures);
        }

        private VkDeviceQueueCreateInfo.Buffer generateQueueMappings() {
            int[] nextQueueIndices = new int[this.queueFamilyProperties.size()];

            for(QueueRequest request : this.queueRequests) {
                int index = nextQueueIndices[request.requestedFamily]++;
                request.assignedIndex = index % this.queueFamilyProperties.get(request.requestedFamily).queueCount();
            }

            int familyCount = 0;
            for(int i : nextQueueIndices) {
                if(i != 0) {
                    familyCount++;
                }
            }

            VkDeviceQueueCreateInfo.Buffer queueCreateInfos = VkDeviceQueueCreateInfo.callocStack(familyCount, this.stack);

            for(int family = 0; family < nextQueueIndices.length; family++) {
                if(nextQueueIndices[family] == 0) {
                    continue;
                }


                FloatBuffer priorities = this.stack.mallocFloat(Math.min(nextQueueIndices[family], this.queueFamilyProperties.get(family).queueCount()));
                while(priorities.hasRemaining()) {
                    priorities.put(1.0f);
                }
                priorities.rewind();

                VkDeviceQueueCreateInfo info = queueCreateInfos.get();
                info.sType(VK10.VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO);
                info.queueFamilyIndex(family);
                info.pQueuePriorities(priorities);
            }

            return queueCreateInfos.rewind();
        }

        private void fulfillQueueRequests(VkDevice device) {
            int queueFamilyCount = this.queueFamilyProperties.size();
            int maxQueueCount = this.queueFamilyProperties.stream().map(VkQueueFamilyProperties::queueCount).max(Comparator.comparingInt(a -> a)).orElse(0);

            VulkanQueue[][] requests = new VulkanQueue[queueFamilyCount][maxQueueCount];

            PointerBuffer pQueue = this.stack.mallocPointer(1);

            for(QueueRequest request : this.queueRequests) {
                int f = request.requestedFamily, i = request.assignedIndex;
                if(requests[f][i] == null) {
                    VK10.vkGetDeviceQueue(device, f, i, pQueue);
                    requests[f][i] = new VulkanQueue(new VkQueue(pQueue.get(0), device), f);
                }

                request.future.complete(requests[f][i]);
            }
        }

        private PointerBuffer generateEnabledExtensionNames() {
            if(this.enabledExtensions.isEmpty()) {
                return null;
            }

            PointerBuffer names = this.stack.mallocPointer(this.enabledExtensions.size());
            for(String extension : this.enabledExtensions) {
                names.put(this.stack.UTF8(extension));
            }

            return names.rewind();
        }

        @Override
        public boolean isApplicationFeatureSupported(NamedID name) {
            ApplicationFeature.Instance feature = this.features.getOrDefault(name, null);
            if(feature == null) {
                return false;
            }

            return feature.isSupported();
        }

        @Override
        public ApplicationFeature.Instance getApplicationFeature(NamedID name) {
            return this.features.getOrDefault(name, null);
        }

        @Override
        public VulkanInstance getInstance() {
            return instance;
        }

        @Override
        public VkPhysicalDevice getPhysicalDevice() {
            return this.physicalDevice;
        }

        @Override
        public VkPhysicalDeviceProperties getPhysicalDeviceProperties() {
            return this.properties;
        }

        @Override
        public VkPhysicalDeviceFeatures getPhysicalDeviceFeatures() {
            return this.availableFeatures;
        }

        @Override
        public boolean isExtensionAvailable(String name) {
            return this.extensionProperties.containsKey(name);
        }

        @Override
        public Map<String, VkExtensionProperties> getAllExtensionProperties() {
            return this.extensionProperties;
        }

        @Override
        public VkExtensionProperties getExtensionProperties(String extension) {
            return this.extensionProperties.getOrDefault(extension, null);
        }

        @Override
        public List<VkQueueFamilyProperties> getQueueFamilyProperties() {
            return queueFamilyProperties;
        }

        @Override
        public List<Integer> findQueueFamilies(int flags, boolean noTransferLimit) {
            List<Integer> ret = new ArrayList<>();
            for(int i = 0; i < this.queueFamilyProperties.size(); i++) {
                VkQueueFamilyProperties properties = this.queueFamilyProperties.get(i);
                if((properties.queueFlags() & flags) == flags) {
                    if(noTransferLimit) {
                        ret.add(i);
                    } else {
                        VkExtent3D granularity = properties.minImageTransferGranularity();
                        if(granularity.width() == 1 && granularity.height() == 1 && granularity.depth() == 1) {
                            ret.add(i);
                        }
                    }
                }
            }
            return ret;
        }

        @Override
        public Future<VulkanQueue> addQueueRequest(int family) {
            assert(this.isBuilding);

            QueueRequest request = new QueueRequest(family);
            this.queueRequests.add(request);
            return request.future;
        }

        @Override
        public void enableExtension(String extension) {
            assert(this.isBuilding);

            this.enabledExtensions.add(extension);
        }

        @Override
        public VkPhysicalDeviceFeatures configureDeviceFeatures() {
            assert(this.isBuilding);

            return this.enabledFeatures;
        }

        private static class QueueRequest {
            private final int requestedFamily;
            private int assignedIndex;
            private final CompletableFuture<VulkanQueue> future;

            private QueueRequest(int family) {
                this.requestedFamily = family;
                this.future = new CompletableFuture<>();
            }
        }
    }
}
