package graphics.kiln.blaze4d.mixin.render;

import com.mojang.blaze3d.systems.RenderSystem;
import com.mojang.blaze3d.vertex.BufferBuilder;
import com.mojang.blaze3d.vertex.BufferUploader;
import com.mojang.blaze3d.vertex.VertexFormat;
import com.mojang.math.Matrix4f;
import graphics.kiln.blaze4d.Blaze4D;
import graphics.kiln.blaze4d.api.B4DShader;
import graphics.kiln.blaze4d.core.types.B4DIndexType;
import graphics.kiln.blaze4d.core.types.B4DMeshData;
import graphics.kiln.blaze4d.core.types.B4DPrimitiveTopology;
import graphics.kiln.blaze4d.core.types.B4DUniformData;
import org.lwjgl.system.MemoryUtil;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.nio.ByteBuffer;
import java.nio.IntBuffer;
import java.util.Objects;

@Mixin(BufferUploader.class)
public class BufferUploaderMixin {


    @Inject(method = "drawWithShader", at = @At("HEAD"))
    private static void drawImmediate(BufferBuilder.RenderedBuffer renderedBuffer, CallbackInfo ci) {
        B4DShader shader = (B4DShader) RenderSystem.getShader();
        if (shader == null) {
            return;
        }

        if (!renderedBuffer.isEmpty()) {
            try(B4DMeshData b4DMeshData = new B4DMeshData()) {
                BufferBuilder.DrawState drawState = renderedBuffer.drawState();
                b4DMeshData.setVertexStride(drawState.format().getVertexSize());
                b4DMeshData.setIndexCount(drawState.indexCount());
                if (drawState.indexType() == VertexFormat.IndexType.SHORT) {
                    b4DMeshData.setIndexType(B4DIndexType.UINT16);
                } else if (drawState.indexType() == VertexFormat.IndexType.INT) {
                    b4DMeshData.setIndexType(B4DIndexType.UINT32);
                } else {
                    return;
                }
                b4DMeshData.setPrimitiveTopology(B4DPrimitiveTopology.fromGLMode(drawState.mode().asGLMode));

                ByteBuffer vertexData = renderedBuffer.vertexBuffer();
                b4DMeshData.setVertexData(MemoryUtil.memAddress(vertexData), vertexData.remaining());

                if(drawState.sequentialIndex()) {
                    IntBuffer indexData = generateSequentialIndices(drawState.mode(), drawState.indexCount());
                    b4DMeshData.setIndexData(MemoryUtil.memAddress(indexData), indexData.remaining() * 4L);
                    b4DMeshData.setIndexType(B4DIndexType.UINT32);
                } else {
                    return;
                    //ByteBuffer indexData = renderedBuffer.indexBuffer();
                    //b4DMeshData.setIndexData(MemoryUtil.memAddress(indexData), indexData.remaining());
                }

                long shaderId = shader.b4dGetShaderId();
                try (B4DUniformData b4DUniformData = new B4DUniformData()) {
                    Matrix4f mat = RenderSystem.getProjectionMatrix();
                    b4DUniformData.setProjectionMatrix(mat.m00, mat.m01, mat.m02, mat.m03, mat.m10, mat.m11, mat.m12, mat.m13, mat.m20, mat.m21, mat.m22, mat.m23, mat.m30, mat.m31, mat.m32, mat.m33);
                    Blaze4D.pushUniform(shaderId, b4DUniformData);
                    mat = RenderSystem.getModelViewMatrix();
                    b4DUniformData.setModelViewMatrix(mat.m00, mat.m01, mat.m02, mat.m03, mat.m10, mat.m11, mat.m12, mat.m13, mat.m20, mat.m21, mat.m22, mat.m23, mat.m30, mat.m31, mat.m32, mat.m33);
                    Blaze4D.pushUniform(shaderId, b4DUniformData);
                }
                Blaze4D.drawImmediate(shader.b4dGetShaderId(), b4DMeshData);
            } catch (Exception e) {
                throw new RuntimeException(e);
            }
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
}
