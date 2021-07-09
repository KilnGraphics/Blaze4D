package me.hydos.rosella.render.texture;

import org.lwjgl.vulkan.VK10;

public enum ImageFormat {
    RGBA(4, 4, VK10.VK_FORMAT_R8G8B8A8_UNORM),
    RGB(3, 3, VK10.VK_FORMAT_R8G8B8_UNORM),
    RG(2, 2, VK10.VK_FORMAT_R8G8_UNORM),
    R(1, 1, VK10.VK_FORMAT_R8_UNORM);

    private final int channels;
    private final int size;
    private final int vkId;

    ImageFormat(int channels, int size, int vkId) {
        this.channels = channels;
        this.size = size;
        this.vkId = vkId;
    }

    public int getChannels() {
        return channels;
    }

    public int getPixelSize() {
        return size;
    }

    public int getVkId() {
        return vkId;
    }

    public static ImageFormat fromVkFormat(int vkFormat) {
        return switch (vkFormat) {
            case VK10.VK_FORMAT_R8G8B8A8_UNORM -> RGBA;
            case VK10.VK_FORMAT_R8G8B8_UNORM -> RGB;
            case VK10.VK_FORMAT_R8G8_UNORM -> RG;
            case VK10.VK_FORMAT_R8_UNORM -> R;
            default -> throw new RuntimeException("Invalid vulkan image format id " + vkFormat);
        };
    }
}
