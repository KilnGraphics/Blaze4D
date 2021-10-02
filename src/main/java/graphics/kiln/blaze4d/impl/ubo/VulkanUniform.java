package graphics.kiln.blaze4d.impl.ubo;

/**
 * Stores extra information within Minecraft's UBOs.
 */
public interface VulkanUniform {
    void writeLocation(long address);

    int alignOffset(int currentOffset);
}
