package me.hydos.rosella.render.texture;

import org.lwjgl.vulkan.VK10;

public enum ImageFormat {
    RGBA(4, 4),
    RGB(3, 3),
    RG(2, 2),
    R(1, 1);

    private final int channels;
    private final int size;

    ImageFormat(int channels, int size) {
        this.channels = channels;
        this.size = size;
    }

    public int getChannels() {
        return channels;
    }

    public int getPixelSize() {
        return size;
    }

    public static ImageFormat fromVkFormat(int vkFormat) {
        return switch (vkFormat) {
            case VK10.VK_FORMAT_R8G8B8A8_UNORM, VK10.VK_FORMAT_R8G8B8A8_SRGB -> RGBA;
            case VK10.VK_FORMAT_R8G8B8_UNORM -> RGB;
            case VK10.VK_FORMAT_R8G8_UNORM -> RG;
            case VK10.VK_FORMAT_R8_UNORM -> R;
            default -> throw new RuntimeException("Invalid vulkan image format id " + vkFormat);
        };
    }
}
