package me.hydos.blaze4d.api.shader;

import java.nio.ByteBuffer;

public interface VulkanUniformBuffer {
    void writeLocation(ByteBuffer buffer);
}
