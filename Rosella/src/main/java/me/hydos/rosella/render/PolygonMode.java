package me.hydos.rosella.render;

import org.lwjgl.vulkan.NVFillRectangle;
import org.lwjgl.vulkan.VK10;

public enum PolygonMode {
    FILL(VK10.VK_POLYGON_MODE_FILL),
    LINE(VK10.VK_POLYGON_MODE_LINE), // TODO: have a wireframe feature and use this mode
    POINT(VK10.VK_POLYGON_MODE_POINT),
    NV_FILL_RECTANGLE(NVFillRectangle.VK_POLYGON_MODE_FILL_RECTANGLE_NV);

    private final int vkId;

    PolygonMode(int vkId) {
        this.vkId = vkId;
    }

    public int getVkId() {
        return vkId;
    }
}
