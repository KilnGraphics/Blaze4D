package graphics.kiln.blaze4d.mixin.render;

import com.mojang.blaze3d.systems.RenderSystem;
import com.mojang.blaze3d.vertex.*;
import graphics.kiln.blaze4d.Blaze4D;
import graphics.kiln.blaze4d.api.B4DShader;
import graphics.kiln.blaze4d.api.B4DVertexBuffer;
import graphics.kiln.blaze4d.core.types.B4DMeshData;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Turns out, Minecraft uses this class for world rendering. when a part of the world is to be rendered, the buffer will be cleared and replaced with just the sky. this will then be uploaded to a {@link VertexBuffer} and then cleared again for the rest of the game to render.
 */
@Mixin(VertexBuffer.class)
public class VertexBufferMixin implements B4DVertexBuffer {

    private long staticMeshId = 0L;

    private Integer currentImmediate = null;

    /**
     * @author Blaze4D
     * @reason Allow for uploading Vertex Buffers
     */
    @Inject(method="upload", at = @At("HEAD"))
    private void uploadBuffer(BufferBuilder.RenderedBuffer renderedBuffer, CallbackInfo ci) {
    }

    @Inject(method="draw", at = @At("HEAD"))
    private void draw(CallbackInfo ci) {
        if(this.currentImmediate != null) {
            if (RenderSystem.getShader() != null) {
                Blaze4D.drawImmediate(((B4DShader) RenderSystem.getShader()).b4dGetShaderId(), this.currentImmediate);
                this.currentImmediate = null;
            }
        }
    }
//
//    /**
//     * @author Blaze4D
//     * @reason Allows rendering things such as the sky.
//     */
//    @Overwrite
//    public void _drawWithShader(com.mojang.math.Matrix4f mcModelViewMatrix, com.mojang.math.Matrix4f mcProjectionMatrix, ShaderInstance shader) {
//        GlobalRenderSystem.updateUniforms(shader, mcModelViewMatrix, mcProjectionMatrix);
//        callWrapperRender(shader);
//    }
//
//
//    @Unique
//    private void callWrapperRender(ShaderInstance mcShader) {
//        RawShaderProgram rawProgram = GlobalRenderSystem.SHADER_PROGRAM_MAP.get(mcShader.getId());
//        ShaderProgram rosellaShaderProgram = Blaze4D.rosella.common.shaderManager.getOrCreateShader(rawProgram);
//        wrapper.render(rosellaShaderProgram, GlobalRenderSystem.getShaderUbo(mcShader));
//    }
//
    @Inject(method = "close", at = @At("HEAD"), cancellable = true)
    private void close(CallbackInfo ci) {
    }

    @Override
    public void setImmediateData(Integer data) {
        this.currentImmediate = data;
    }
}
