package me.hydos.rosella.render.material.state;

import org.jetbrains.annotations.NotNull;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.vulkan.VK10;
import org.lwjgl.vulkan.VkExtent2D;
import org.lwjgl.vulkan.VkOffset2D;
import org.lwjgl.vulkan.VkPipelineColorBlendAttachmentState;
import org.lwjgl.vulkan.VkPipelineColorBlendStateCreateInfo;
import org.lwjgl.vulkan.VkPipelineDepthStencilStateCreateInfo;
import org.lwjgl.vulkan.VkPipelineRasterizationStateCreateInfo;
import org.lwjgl.vulkan.VkRect2D;

import java.util.Objects;

public class StateInfo {
    private int colorMask;
    private boolean depthMask;

    private boolean scissorEnabled;
    private int scissorX;
    private int scissorY;
    private int scissorWidth;
    private int scissorHeight;

    private boolean stencilEnabled;

    private boolean blendEnabled;
    private int srcColorBlendFactor;
    private int dstColorBlendFactor;
    private int srcAlphaBlendFactor;
    private int dstAlphaBlendFactor;
    private int blendOp;

    private boolean cullEnabled;

    private boolean depthTestEnabled;
    private int depthCompareOp;

    private boolean colorLogicOpEnabled;
    private int colorLogicOp;

    private float lineWidth;

    public StateInfo(int colorMask, boolean depthMask, boolean scissorEnabled, int scissorX, int scissorY, int scissorWidth, int scissorHeight, boolean stencilEnabled, boolean blendEnabled, int srcColorBlendFactor, int dstColorBlendFactor, int srcAlphaBlendFactor, int dstAlphaBlendFactor, int blendOp, boolean cullEnabled, boolean depthTestEnabled, int depthCompareOp, boolean colorLogicOpEnabled, int colorLogicOp, float lineWidth) {

        this.colorMask = colorMask;
        this.depthMask = depthMask;
        this.scissorEnabled = scissorEnabled;
        this.scissorX = scissorX;
        this.scissorY = scissorY;
        this.scissorWidth = scissorWidth;
        this.scissorHeight = scissorHeight;
        this.stencilEnabled = stencilEnabled;
        this.blendEnabled = blendEnabled;
        this.srcColorBlendFactor = srcColorBlendFactor;
        this.dstColorBlendFactor = dstColorBlendFactor;
        this.srcAlphaBlendFactor = srcAlphaBlendFactor;
        this.dstAlphaBlendFactor = dstAlphaBlendFactor;
        this.blendOp = blendOp;
        this.cullEnabled = cullEnabled;
        this.depthTestEnabled = depthTestEnabled;
        this.depthCompareOp = depthCompareOp;
        this.colorLogicOpEnabled = colorLogicOpEnabled;
        this.colorLogicOp = colorLogicOp;
        this.lineWidth = lineWidth;
    }

    public StateInfo(StateInfo info) {
        this(info.colorMask, info.depthMask, info.scissorEnabled, info.scissorX, info.scissorY, info.scissorWidth, info.scissorHeight, info.stencilEnabled, info.blendEnabled, info.srcColorBlendFactor, info.dstColorBlendFactor, info.srcAlphaBlendFactor, info.dstAlphaBlendFactor, info.blendOp, info.cullEnabled, info.depthTestEnabled, info.depthCompareOp, info.colorLogicOpEnabled, info.colorLogicOp, info.lineWidth);
    }

    public StateInfo snapshot() {
        return new StateInfo(this);
    }

    public StateInfo setColorMask(int colorMask) {
        this.colorMask = colorMask;
        return this;
    }

    public StateInfo setDepthMask(boolean depthMask) {
        this.depthMask = depthMask;
        return this;
    }

    public StateInfo setScissorEnabled(boolean scissorEnabled) {
        this.scissorEnabled = scissorEnabled;
        return this;
    }

    public StateInfo setScissorX(int scissorX) {
        this.scissorX = scissorX;
        return this;
    }

    public StateInfo setScissorY(int scissorY) {
        this.scissorY = scissorY;
        return this;
    }

    public StateInfo setScissorWidth(int scissorWidth) {
        this.scissorWidth = scissorWidth;
        return this;
    }

    public StateInfo setScissorHeight(int scissorHeight) {
        this.scissorHeight = scissorHeight;
        return this;
    }

    public StateInfo setStencilEnabled(boolean stencilEnabled) {
        this.stencilEnabled = stencilEnabled;
        return this;
    }

    public StateInfo setBlendEnabled(boolean blendEnabled) {
        this.blendEnabled = blendEnabled;
        return this;
    }

    public StateInfo setSrcColorBlendFactor(int srcColorBlendFactor) {
        this.srcColorBlendFactor = srcColorBlendFactor;
        return this;
    }

    public StateInfo setDstColorBlendFactor(int dstColorBlendFactor) {
        this.dstColorBlendFactor = dstColorBlendFactor;
        return this;
    }

    public StateInfo setSrcAlphaBlendFactor(int srcAlphaBlendFactor) {
        this.srcAlphaBlendFactor = srcAlphaBlendFactor;
        return this;
    }

    public StateInfo setDstAlphaBlendFactor(int dstAlphaBlendFactor) {
        this.dstAlphaBlendFactor = dstAlphaBlendFactor;
        return this;
    }

    public StateInfo setBlendOp(int blendOp) {
        this.blendOp = blendOp;
        return this;
    }

    public StateInfo setCullEnabled(boolean cullEnabled) {
        this.cullEnabled = cullEnabled;
        return this;
    }

    public StateInfo setDepthTestEnabled(boolean depthTestEnabled) {
        this.depthTestEnabled = depthTestEnabled;
        return this;
    }

    public StateInfo setDepthCompareOp(int depthCompareOp) {
        this.depthCompareOp = depthCompareOp;
        return this;
    }

    public StateInfo setColorLogicOpEnabled(boolean colorLogicOpEnabled) {
        this.colorLogicOpEnabled = colorLogicOpEnabled;
        return this;
    }

    public StateInfo setColorLogicOp(int colorLogicOp) {
        this.colorLogicOp = colorLogicOp;
        return this;
    }

    public StateInfo setLineWidth(float lineWidth) {
        this.lineWidth = lineWidth;
        return this;
    }

    public int getColorMask() {
        return colorMask;
    }

    public boolean isDepthMask() {
        return depthMask;
    }

    public boolean isScissorEnabled() {
        return scissorEnabled;
    }

    public int getScissorX() {
        return scissorX;
    }

    public int getScissorY() {
        return scissorY;
    }

    public int getScissorWidth() {
        return scissorWidth;
    }

    public int getScissorHeight() {
        return scissorHeight;
    }

    public boolean isStencilEnabled() {
        return stencilEnabled;
    }

    public boolean isBlendEnabled() {
        return blendEnabled;
    }

    public int getSrcColorBlendFactor() {
        return srcColorBlendFactor;
    }

    public int getDstColorBlendFactor() {
        return dstColorBlendFactor;
    }

    public int getSrcAlphaBlendFactor() {
        return srcAlphaBlendFactor;
    }

    public int getDstAlphaBlendFactor() {
        return dstAlphaBlendFactor;
    }

    public int getBlendOp() {
        return blendOp;
    }

    public boolean isCullEnabled() {
        return cullEnabled;
    }

    public boolean isDepthTestEnabled() {
        return depthTestEnabled;
    }

    public int getDepthCompareOp() {
        return depthCompareOp;
    }

    public boolean isColorLogicOpEnabled() {
        return colorLogicOpEnabled;
    }

    public int getColorLogicOp() {
        return colorLogicOp;
    }

    public float getLineWidth() {
        return lineWidth;
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        StateInfo stateInfo = (StateInfo) o;
        return colorMask == stateInfo.colorMask && depthMask == stateInfo.depthMask && scissorEnabled == stateInfo.scissorEnabled && scissorX == stateInfo.scissorX && scissorY == stateInfo.scissorY && scissorWidth == stateInfo.scissorWidth && scissorHeight == stateInfo.scissorHeight && stencilEnabled == stateInfo.stencilEnabled && blendEnabled == stateInfo.blendEnabled && srcColorBlendFactor == stateInfo.srcColorBlendFactor && dstColorBlendFactor == stateInfo.dstColorBlendFactor && srcAlphaBlendFactor == stateInfo.srcAlphaBlendFactor && dstAlphaBlendFactor == stateInfo.dstAlphaBlendFactor && blendOp == stateInfo.blendOp && cullEnabled == stateInfo.cullEnabled && depthTestEnabled == stateInfo.depthTestEnabled && depthCompareOp == stateInfo.depthCompareOp && colorLogicOpEnabled == stateInfo.colorLogicOpEnabled && colorLogicOp == stateInfo.colorLogicOp && Float.compare(stateInfo.lineWidth, lineWidth) == 0;
    }

    @Override
    public int hashCode() {
        return Objects.hash(colorMask, depthMask, scissorEnabled, scissorX, scissorY, scissorWidth, scissorHeight, stencilEnabled, blendEnabled, srcColorBlendFactor, dstColorBlendFactor, srcAlphaBlendFactor, dstAlphaBlendFactor, blendOp, cullEnabled, depthTestEnabled, depthCompareOp, colorLogicOpEnabled, colorLogicOp, lineWidth);
    }

    @NotNull
    public VkRect2D.Buffer getExtent() {
        return VkRect2D.callocStack(1)
                .offset(VkOffset2D.callocStack().set(getScissorX(), getScissorY()))
                .extent(VkExtent2D.callocStack().set(getScissorWidth(), getScissorHeight()));
    }

    @NotNull
    public VkPipelineRasterizationStateCreateInfo getRasterizationStateCreateInfo(int polygonMode) {
        return VkPipelineRasterizationStateCreateInfo.callocStack()
                .sType(VK10.VK_STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO)
                .depthClampEnable(true)
                .rasterizerDiscardEnable(false)
                .polygonMode(polygonMode)
                .lineWidth(getLineWidth())
                .cullMode(isCullEnabled() ? VK10.VK_CULL_MODE_BACK_BIT : VK10.VK_CULL_MODE_NONE)
                .frontFace(VK10.VK_FRONT_FACE_COUNTER_CLOCKWISE) // TODO: try messing with this
                .depthBiasEnable(false);
    }

    @NotNull
    public VkPipelineDepthStencilStateCreateInfo getPipelineDepthStencilStateCreateInfo() {
        return VkPipelineDepthStencilStateCreateInfo.callocStack()
                .sType(VK10.VK_STRUCTURE_TYPE_PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO)
                .depthTestEnable(isDepthTestEnabled())
                .depthWriteEnable(isDepthMask())
                .depthCompareOp(getDepthCompareOp())
                .depthBoundsTestEnable(true)
                .minDepthBounds(0f)
                .maxDepthBounds(1.0f)
                .stencilTestEnable(isStencilEnabled());
    }

    @NotNull
    public VkPipelineColorBlendAttachmentState.Buffer getPipelineColorBlendAttachmentStates() {
        return VkPipelineColorBlendAttachmentState.callocStack(1)
                .colorWriteMask(getColorMask())
                .blendEnable(isBlendEnabled())
                .srcColorBlendFactor(getSrcColorBlendFactor())
                .dstColorBlendFactor(getDstColorBlendFactor())
                .colorBlendOp(getBlendOp())
                .srcAlphaBlendFactor(getSrcAlphaBlendFactor())
                .dstAlphaBlendFactor(getDstAlphaBlendFactor())
                .alphaBlendOp(getBlendOp());
    }

    @NotNull
    public VkPipelineColorBlendStateCreateInfo getPipelineColorBlendStateCreateInfo(VkPipelineColorBlendAttachmentState.Buffer colourBlendAttachment) {
        return VkPipelineColorBlendStateCreateInfo.callocStack()
                .sType(VK10.VK_STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO)
                .logicOpEnable(isColorLogicOpEnabled())
                .logicOp(getColorLogicOp())
                .pAttachments(colourBlendAttachment)
                .blendConstants(MemoryStack.stackGet().floats(0.0f, 0.0f, 0.0f, 0.0f));
    }


}
