package me.hydos.rosella.render.texture;

import org.lwjgl.vulkan.VK10;

public enum WrapMode {

    REPEAT(VK10.VK_SAMPLER_ADDRESS_MODE_REPEAT),
    MIRRORED_REPEAT(VK10.VK_SAMPLER_ADDRESS_MODE_MIRRORED_REPEAT),
    CLAMP_TO_EDGE(VK10.VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE),
    CLAMP_TO_BORDER(VK10.VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_BORDER);

    public final int vkType;

    WrapMode(int vkType) {
        this.vkType = vkType;
    }
}
