package me.hydos.rosella.device.init;

import org.lwjgl.vulkan.VK10;
import org.lwjgl.vulkan.VK11;
import org.lwjgl.vulkan.VK12;

public enum VulkanVersion {
    VULKAN_1_0(VK10.VK_API_VERSION_1_0),
    VULKAN_1_1(VK11.VK_API_VERSION_1_1),
    VULKAN_1_2(VK12.VK_API_VERSION_1_2);

    private final int versionNumber;

    VulkanVersion(int versionNumber) {
        this.versionNumber = versionNumber;
    }

    public int getVersionNumber() {
        return this.versionNumber;
    }

    @Override
    public String toString() {
        int major = VK10.VK_VERSION_MAJOR(this.versionNumber);
        int minor = VK10.VK_VERSION_MINOR(this.versionNumber);
        int patch = VK10.VK_VERSION_PATCH(this.versionNumber);

        return "" + major + "." + minor + "." + patch;
    }

    /**
     * Returns the vulkan version that matches the provided version number.
     * If the version number is higher than any known version returns the highest known version.
     *
     * @param versionNumber The version number
     * @return A version object matching the version number.
     */
    public static VulkanVersion fromVersionNumber(int versionNumber) {
        if(versionNumber < VULKAN_1_1.getVersionNumber()) {
            return VULKAN_1_0;
        }
        if(versionNumber < VULKAN_1_2.getVersionNumber()) {
            return VULKAN_1_1;
        }
        return VULKAN_1_2;
    }
}
