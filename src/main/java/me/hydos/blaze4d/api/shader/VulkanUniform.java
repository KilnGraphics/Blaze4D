package me.hydos.blaze4d.api.shader;

public interface VulkanUniform {
    void writeLocation(long address);

    int alignOffset(int currentOffset);
}
