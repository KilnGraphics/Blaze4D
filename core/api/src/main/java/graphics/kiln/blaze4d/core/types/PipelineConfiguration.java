package graphics.kiln.blaze4d.core.types;

import graphics.kiln.blaze4d.core.natives.PipelineConfigurationNative;
import jdk.incubator.foreign.MemoryAddress;
import jdk.incubator.foreign.MemorySegment;
import jdk.incubator.foreign.ResourceScope;

public class PipelineConfiguration implements AutoCloseable {

    private final ResourceScope resourceScope;
    private final MemorySegment memory;

    public PipelineConfiguration() {
        this.resourceScope = ResourceScope.newSharedScope();
        this.memory = MemorySegment.allocateNative(PipelineConfigurationNative.LAYOUT, this.resourceScope);
    }

    public void setDepthTestEnable(boolean enable) {
        PipelineConfigurationNative.DEPTH_TEST_ENABLE_HANDLE.set(this.memory, enable ? 1 : 0);
    }

    public boolean getDepthTestEnable() {
        return ((int) PipelineConfigurationNative.DEPTH_TEST_ENABLE_HANDLE.get(this.memory)) != 0;
    }

    public void setDepthCompareOp(CompareOp op) {
        PipelineConfigurationNative.DEPTH_COMPARE_OP_HANDLE.set(this.memory, op.getValue());
    }

    public CompareOp getDepthCompareOp() {
        return CompareOp.fromValue((int) PipelineConfigurationNative.DEPTH_COMPARE_OP_HANDLE.get(this.memory));
    }

    public void setDepthWriteEnable(boolean enable) {
        PipelineConfigurationNative.DEPTH_WRITE_ENABLE_HANDLE.set(this.memory, enable ? 1 : 0);
    }

    public boolean getDepthWriteEnable() {
        return ((int) PipelineConfigurationNative.DEPTH_WRITE_ENABLE_HANDLE.get(this.memory)) != 0;
    }

    public void setBlendEnable(boolean enable) {
        PipelineConfigurationNative.BLEND_ENABLE_HANDLE.set(this.memory, enable ? 1 : 0);
    }

    public boolean getBlendEnable() {
        return ((int) PipelineConfigurationNative.BLEND_ENABLE_HANDLE.get(this.memory)) != 0;
    }

    public void setBlendColorOp(BlendOp op) {
        PipelineConfigurationNative.BLEND_COLOR_OP_HANDLE.set(this.memory, op.getValue());
    }

    public BlendOp getBlendColorOp() {
        return BlendOp.fromValue((int) PipelineConfigurationNative.BLEND_COLOR_OP_HANDLE.get(this.memory));
    }

    public void setBlendColorSrcFactor(BlendFactor factor) {
        PipelineConfigurationNative.BLEND_COLOR_SRC_FACTOR_HANDLE.set(this.memory, factor.getValue());
    }

    public BlendFactor getBlendColorSrcFactor() {
        return BlendFactor.fromValue((int) PipelineConfigurationNative.BLEND_COLOR_SRC_FACTOR_HANDLE.get(this.memory));
    }

    public void setBlendColorDstFactor(BlendFactor factor) {
        PipelineConfigurationNative.BLEND_COLOR_DST_FACTOR_HANDLE.set(this.memory, factor.getValue());
    }

    public BlendFactor getBlendColorDstFactor() {
        return BlendFactor.fromValue((int) PipelineConfigurationNative.BLEND_COLOR_DST_FACTOR_HANDLE.get(this.memory));
    }

    public void setBlendAlphaOp(BlendOp op) {
        PipelineConfigurationNative.BLEND_ALPHA_OP_HANDLE.set(this.memory, op.getValue());
    }

    public BlendOp getBlendAlphaOp() {
        return BlendOp.fromValue((int) PipelineConfigurationNative.BLEND_ALPHA_OP_HANDLE.get(this.memory));
    }

    public void setBlendAlphaSrcFactor(BlendFactor factor) {
        PipelineConfigurationNative.BLEND_ALPHA_SRC_FACTOR_HANDLE.set(this.memory, factor.getValue());
    }

    public BlendFactor getBlendAlphaSrcFactor() {
        return BlendFactor.fromValue((int) PipelineConfigurationNative.BLEND_ALPHA_SRC_FACTOR_HANDLE.get(this.memory));
    }

    public void setBlendAlphaDstFactor(BlendFactor factor) {
        PipelineConfigurationNative.BLEND_ALPHA_DST_FACTOR_HANDLE.set(this.memory, factor.getValue());
    }

    public BlendFactor getBlendAlphaDstFactor() {
        return BlendFactor.fromValue((int) PipelineConfigurationNative.BLEND_ALPHA_DST_FACTOR_HANDLE.get(this.memory));
    }

    public MemoryAddress getAddress() {
        return this.memory.address();
    }

    @Override
    public void close() throws Exception {
        this.resourceScope.close();
    }
}
