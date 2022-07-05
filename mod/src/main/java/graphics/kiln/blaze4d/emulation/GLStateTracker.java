/**
 * Tracks opengl state for vulkan emulation.
 */

package graphics.kiln.blaze4d.emulation;

import graphics.kiln.blaze4d.core.types.BlendFactor;
import graphics.kiln.blaze4d.core.types.BlendOp;
import graphics.kiln.blaze4d.core.types.CompareOp;
import graphics.kiln.blaze4d.core.types.PipelineConfiguration;

public class GLStateTracker {
    public static GLStateTracker INSTANCE = new GLStateTracker();

    private final PipelineConfiguration pipelineConfiguration;

    public GLStateTracker() {
        this.pipelineConfiguration = new PipelineConfiguration();
        this.pipelineConfiguration.setDepthTestEnable(false);
        this.pipelineConfiguration.setDepthCompareOp(CompareOp.LESS);
        this.pipelineConfiguration.setDepthWriteEnable(false);
        this.pipelineConfiguration.setBlendEnable(false);
        this.pipelineConfiguration.setBlendColorOp(BlendOp.ADD);
        this.pipelineConfiguration.setBlendColorSrcFactor(BlendFactor.ONE);
        this.pipelineConfiguration.setBlendColorDstFactor(BlendFactor.ZERO);
        this.pipelineConfiguration.setBlendAlphaOp(BlendOp.ADD);
        this.pipelineConfiguration.setBlendAlphaSrcFactor(BlendFactor.ONE);
        this.pipelineConfiguration.setBlendAlphaDstFactor(BlendFactor.ZERO);
    }

    public PipelineConfiguration getPipelineConfiguration() {
        return this.pipelineConfiguration;
    }

    public void setDepthTest(boolean enable) {
        this.pipelineConfiguration.setDepthTestEnable(enable);
    }

    public void setDepthFunc(int glFunc) {
        this.setDepthFunc(CompareOp.fromGlDepthFunc(glFunc));
    }

    public void setDepthFunc(CompareOp op) {
        this.pipelineConfiguration.setDepthCompareOp(op);
    }

    public void setDepthMask(boolean enable) {
        this.pipelineConfiguration.setDepthWriteEnable(enable);
    }

    public void setBlendFunc(int srcFunc, int dstFunc) {
        this.setBlendFunc(BlendFactor.fromGlBlendFunc(srcFunc), BlendFactor.fromGlBlendFunc(dstFunc));
    }

    public void setBlendFunc(BlendFactor src, BlendFactor dst) {
        this.setBlendFuncSeparate(src, dst, src, dst);
    }

    public void setBlendFuncSeparate(int colorSrcFunc, int colorDstFunc, int alphaSrcFunc, int alphaDstFunc) {
        this.setBlendFuncSeparate(BlendFactor.fromGlBlendFunc(colorSrcFunc), BlendFactor.fromGlBlendFunc(colorDstFunc), BlendFactor.fromGlBlendFunc(alphaSrcFunc), BlendFactor.fromGlBlendFunc(alphaDstFunc));
    }

    public void setBlendFuncSeparate(BlendFactor colorSrc, BlendFactor colorDst, BlendFactor alphaSrc, BlendFactor alphaDst) {
        this.pipelineConfiguration.setBlendColorSrcFactor(colorSrc);
        this.pipelineConfiguration.setBlendColorDstFactor(colorDst);
        this.pipelineConfiguration.setBlendAlphaSrcFactor(alphaSrc);
        this.pipelineConfiguration.setBlendAlphaDstFactor(alphaDst);
    }

    public void setBlendEquation(int glEquation) {
        this.setBlendEquation(BlendOp.fromGlBlendEquation(glEquation));
    }

    public void setBlendEquation(BlendOp op) {
        this.pipelineConfiguration.setBlendColorOp(op);
        this.pipelineConfiguration.setBlendAlphaOp(op);
    }
}
