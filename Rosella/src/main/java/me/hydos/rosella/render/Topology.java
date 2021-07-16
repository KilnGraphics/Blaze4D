package me.hydos.rosella.render;

import org.lwjgl.vulkan.VK10;

public enum Topology {

    TRIANGLES(VK10.VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST),
    TRIANGLE_STRIP(VK10.VK_PRIMITIVE_TOPOLOGY_TRIANGLE_STRIP),
    TRIANGLE_FAN(VK10.VK_PRIMITIVE_TOPOLOGY_TRIANGLE_FAN),
    LINE_LIST(VK10.VK_PRIMITIVE_TOPOLOGY_LINE_LIST),
    LINE_STRIP(VK10.VK_PRIMITIVE_TOPOLOGY_LINE_STRIP);

    public final int vkType;

    Topology(int vkType) {
        this.vkType = vkType;
    }
}
