package graphics.kiln.blaze4d.mixin.render;

import com.mojang.blaze3d.systems.RenderSystem;
import com.mojang.blaze3d.vertex.*;
import graphics.kiln.blaze4d.Blaze4D;
import graphics.kiln.blaze4d.api.B4DShader;
import graphics.kiln.blaze4d.api.B4DVertexBuffer;
import graphics.kiln.blaze4d.core.GlobalMesh;
import graphics.kiln.blaze4d.core.types.B4DIndexType;
import graphics.kiln.blaze4d.core.types.B4DMeshData;
import graphics.kiln.blaze4d.core.types.B4DPrimitiveTopology;
import org.lwjgl.system.MemoryUtil;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.nio.ByteBuffer;
import java.nio.IntBuffer;

/**
 * Turns out, Minecraft uses this class for world rendering. when a part of the world is to be rendered, the buffer will be cleared and replaced with just the sky. this will then be uploaded to a {@link VertexBuffer} and then cleared again for the rest of the game to render.
 */
@Mixin(VertexBuffer.class)
public class VertexBufferMixin implements B4DVertexBuffer {

    @Final
    private B4DMeshData meshData = new B4DMeshData();
    private GlobalMesh globalMesh = null;

    private Integer currentImmediate = null;

    /**
     * @author Blaze4D
     * @reason Allow for uploading Vertex Buffers
     */
    @Inject(method="upload", at = @At("HEAD"))
    private void uploadBuffer(BufferBuilder.RenderedBuffer renderedBuffer, CallbackInfo ci) {
        if (this.globalMesh != null) {
            try {
                this.globalMesh.close();
            } catch (Exception e) {
                throw new RuntimeException(e);
            }
            this.globalMesh = null;
        }

        if (!renderedBuffer.isEmpty()) {
            BufferBuilder.DrawState drawState = renderedBuffer.drawState();
            this.meshData.setVertexStride(drawState.format().getVertexSize());
            this.meshData.setIndexCount(drawState.indexCount());
            this.meshData.setPrimitiveTopology(B4DPrimitiveTopology.fromGLMode(drawState.mode().asGLMode));

            ByteBuffer vertexData = renderedBuffer.vertexBuffer();
            this.meshData.setVertexData(MemoryUtil.memAddress(vertexData), vertexData.remaining());

            if(drawState.sequentialIndex()) {
                this.meshData.setIndexType(B4DIndexType.UINT32);

                IntBuffer indexData = generateSequentialIndices(drawState.mode(), drawState.indexCount());
                this.meshData.setIndexData(MemoryUtil.memAddress(indexData), indexData.remaining() * 4L);
            } else {
                if (drawState.indexType() == VertexFormat.IndexType.SHORT) {
                    this.meshData.setIndexType(B4DIndexType.UINT16);
                } else if (drawState.indexType() == VertexFormat.IndexType.INT) {
                    this.meshData.setIndexType(B4DIndexType.UINT32);
                } else {
                    return;
                }

                ByteBuffer indexData = renderedBuffer.indexBuffer();
                this.meshData.setIndexData(MemoryUtil.memAddress(indexData), indexData.remaining());
            }

            this.globalMesh = Blaze4D.core.createGlobalMesh(this.meshData);
        }
    }

    private static IntBuffer generateSequentialIndices(VertexFormat.Mode mode, int indexCount) {
        IntBuffer indices = MemoryUtil.memAllocInt(indexCount);
        indices.limit(indexCount);
        switch(mode) {
            case QUADS -> {
                for (int i = 0; i < indexCount / 6 * 4; i += 4) {
                    indices.put(i);
                    indices.put(i + 1);
                    indices.put(i + 2);
                    indices.put(i + 2);
                    indices.put(i + 3);
                    indices.put(i);
                }
            }
            default -> {
                for (int i = 0; i < indexCount; i++) {
                    indices.put(i);
                }
            }
        }
        return indices.rewind();
    }

    @Inject(method="draw", at = @At("HEAD"))
    private void draw(CallbackInfo ci) {
        if(this.currentImmediate != null) {
            if (RenderSystem.getShader() != null) {
                Blaze4D.drawImmediate(this.currentImmediate, ((B4DShader) RenderSystem.getShader()).b4dGetShaderId());
                this.currentImmediate = null;
            }
        } else if (this.globalMesh != null) {
            if (RenderSystem.getShader() != null) {
                Blaze4D.drawGlobal(this.globalMesh, ((B4DShader) RenderSystem.getShader()).b4dGetShaderId());
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
    private void close(CallbackInfo ci) throws Exception {
        if (this.globalMesh != null) {
            this.globalMesh.close();
            this.globalMesh = null;
        }
        this.meshData.close();
    }

    @Override
    public void setImmediateData(Integer data) {
        this.currentImmediate = data;
    }
}
