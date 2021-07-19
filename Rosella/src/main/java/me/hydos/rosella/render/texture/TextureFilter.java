package me.hydos.rosella.render.texture;

import org.lwjgl.vulkan.VK10;

public enum TextureFilter {

    NEAREST(VK10.VK_FILTER_NEAREST),
    LINEAR(VK10.VK_FILTER_LINEAR);

    public final int vkType;

    TextureFilter(int vkType) {
        this.vkType = vkType;
    }
}
