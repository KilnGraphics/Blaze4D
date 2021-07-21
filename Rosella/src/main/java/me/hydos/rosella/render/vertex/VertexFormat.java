package me.hydos.rosella.render.vertex;

import org.lwjgl.system.MemoryStack;
import org.lwjgl.vulkan.VK10;
import org.lwjgl.vulkan.VkPipelineVertexInputStateCreateInfo;
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

    public VkPipelineVertexInputStateCreateInfo getPipelineVertexInputStateCreateInfo() {
        MemoryStack stack = MemoryStack.stackGet();
        return VkPipelineVertexInputStateCreateInfo.callocStack(stack)
                .sType(VK10.VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO)
                .pVertexBindingDescriptions(getVkBindings())
                .pVertexAttributeDescriptions(getVkAttributes());
    }
}
