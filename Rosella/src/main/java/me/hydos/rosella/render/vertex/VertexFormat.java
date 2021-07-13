package me.hydos.rosella.render.vertex;

import org.lwjgl.vulkan.VK10;
import org.lwjgl.vulkan.VkVertexInputAttributeDescription;
import org.lwjgl.vulkan.VkVertexInputBindingDescription;

public class VertexFormat {
    private final VertexFormatElement[] elements;
    private final VkVertexInputAttributeDescription.Buffer vkAttributes;
    private final VkVertexInputBindingDescription.Buffer vkBindings;
    private final int size;

    VertexFormat(VertexFormatElement... elements) {
        this.elements = elements;

        int correctedLength = 0;
        for (VertexFormatElement vertexFormatElement : elements) {
            if (vertexFormatElement.vkType() != VertexFormatElements.VK_FORMAT_PADDING) {
                correctedLength++;
            }
        }

        this.vkAttributes = VkVertexInputAttributeDescription.callocStack(correctedLength);

        int offset = 0;
        int elementIdx = 0;
        for (VertexFormatElement element : elements) {
            if (element.vkType() != VertexFormatElements.VK_FORMAT_PADDING) {
                vkAttributes.get(elementIdx)
                        .binding(0)
                        .location(elementIdx)
                        .format(element.vkType())
                        .offset(offset);
                elementIdx++;
            }
            offset += element.byteLength();
        }
        vkAttributes.rewind();

        System.out.println("Vertex Attributes: ");
        for (int i = 0; i < correctedLength; i++) {
            System.out.printf("\tIndex %d: location %d, binding: %d, format: %d, offset: %d\n", i, vkAttributes.get(i).location(), vkAttributes.get(i).binding(), vkAttributes.get(i).format(), vkAttributes.get(i).offset());
        }

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
