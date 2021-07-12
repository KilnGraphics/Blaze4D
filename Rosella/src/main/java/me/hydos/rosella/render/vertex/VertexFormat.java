package me.hydos.rosella.render.vertex;

import org.lwjgl.vulkan.VK10;
import org.lwjgl.vulkan.VkVertexInputAttributeDescription;
import org.lwjgl.vulkan.VkVertexInputBindingDescription;

public class  VertexFormat {
    private final VertexFormatElement[] elements;
    private final VkVertexInputAttributeDescription.Buffer vkAttributes;
    private final VkVertexInputBindingDescription.Buffer vkBindings;
    private final int size;

    VertexFormat(VertexFormatElement... elements) {
        this.elements = elements;

        this.vkAttributes = VkVertexInputAttributeDescription.callocStack(elements.length);

        int offset = 0;
        for (int idx = 0; idx < elements.length; idx++) {
            VertexFormatElement element = elements[idx];
            if (element.vkType() != VertexFormatElements.VK_FORMAT_PADDING) { // TODO: You have an empty attribute here if you have padding
                vkAttributes.get(idx)
                        .binding(0)
                        .location(idx)
                        .format(element.vkType())
                        .offset(offset);
            }
            offset += element.byteLength();
        }
        vkAttributes.rewind();

        this.size = offset;

        this.vkBindings = VkVertexInputBindingDescription.callocStack(1)
                .binding(0)
                .stride(size)
                .inputRate(VK10.VK_VERTEX_INPUT_RATE_VERTEX);
    }

    public VkVertexInputAttributeDescription.Buffer getVkAttributes() {
        return vkAttributes;
    }

    public VkVertexInputBindingDescription.Buffer getVkBindings() {
        return vkBindings;
    }

    public int getSize() {
        return size;
    }

    public VertexFormatElement[] getElements() {
        return elements;
    }

}
