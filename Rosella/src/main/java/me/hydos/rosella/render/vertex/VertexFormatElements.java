package me.hydos.rosella.render.vertex;

import it.unimi.dsi.fastutil.objects.Object2ObjectOpenHashMap;
import org.lwjgl.vulkan.VK10;

import java.util.Map;

public class VertexFormatElements {

    private static final Map<VertexFormatElement, VertexFormatElement> ELEMENTS_POOL = new Object2ObjectOpenHashMap<>();

    public static final VertexFormatElement POSITION = getElement(VK10.VK_FORMAT_R32G32B32_SFLOAT, VertexFormatElement.DataType.FLOAT.getByteLength() * 3);
    public static final VertexFormatElement NORMAL = getElement(VK10.VK_FORMAT_R8G8B8_UINT, VertexFormatElement.DataType.UINT.getByteLength() * 3);
    public static final VertexFormatElement COLOR3ub = getElement(VK10.VK_FORMAT_R8G8B8_UNORM, VertexFormatElement.DataType.UBYTE.getByteLength() * 3);
    public static final VertexFormatElement COLOR4ub = getElement(VK10.VK_FORMAT_R8G8B8A8_UNORM, VertexFormatElement.DataType.UBYTE.getByteLength() * 4);
    public static final VertexFormatElement COLOR3f = getElement(VK10.VK_FORMAT_R32G32B32_SFLOAT, VertexFormatElement.DataType.FLOAT.getByteLength() * 3);
    public static final VertexFormatElement COLOR4f = getElement(VK10.VK_FORMAT_R32G32B32A32_SFLOAT, VertexFormatElement.DataType.FLOAT.getByteLength() * 4);
    public static final VertexFormatElement UVs = getElement(VK10.VK_FORMAT_R16G16_SINT, VertexFormatElement.DataType.SHORT.getByteLength() * 2);
    public static final VertexFormatElement UVf = getElement(VK10.VK_FORMAT_R32G32_SFLOAT, VertexFormatElement.DataType.FLOAT.getByteLength() * 2);
    public static final VertexFormatElement GENERICb = getElement(VK10.VK_FORMAT_R8_SINT, VertexFormatElement.DataType.BYTE.getByteLength());
    public static final VertexFormatElement GENERICf = getElement(VK10.VK_FORMAT_R32_SFLOAT, VertexFormatElement.DataType.FLOAT.getByteLength());

    // makes sure we don't waste a ton of memory with duplicates that get caught in the materials cache
    public static VertexFormatElement getElement(int vkId, int size) {
        VertexFormatElement newElement = new VertexFormatElement(vkId, size);
        return ELEMENTS_POOL.computeIfAbsent(newElement, e -> newElement);
    }
}
