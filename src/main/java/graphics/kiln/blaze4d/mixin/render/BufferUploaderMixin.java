package graphics.kiln.blaze4d.mixin.render;

import com.mojang.blaze3d.systems.RenderSystem;
import com.mojang.blaze3d.vertex.BufferBuilder;
import com.mojang.blaze3d.vertex.BufferUploader;
import com.mojang.blaze3d.vertex.VertexBuffer;
import com.mojang.blaze3d.vertex.VertexFormat;
import graphics.kiln.blaze4d.Blaze4D;
import graphics.kiln.blaze4d.api.B4DShader;
import graphics.kiln.blaze4d.api.B4DVertexBuffer;
import graphics.kiln.blaze4d.core.types.B4DIndexType;
import graphics.kiln.blaze4d.core.types.B4DMeshData;
import graphics.kiln.blaze4d.core.types.B4DPrimitiveTopology;
import org.lwjgl.system.MemoryUtil;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.nio.ByteBuffer;
import java.nio.IntBuffer;

@Mixin(BufferUploader.class)
public class BufferUploaderMixin {

    @Inject(method = "upload", at = @At("RETURN"))
    private static void prepareImmediate(BufferBuilder.RenderedBuffer renderedBuffer, CallbackInfoReturnable<VertexBuffer> ci) {
        if(ci.getReturnValue() != null) {
            drawImmediateBuffer(renderedBuffer, (B4DVertexBuffer) ci.getReturnValue());
        }
    }

    private static void drawImmediateBuffer(BufferBuilder.RenderedBuffer renderedBuffer, B4DVertexBuffer target) {
        B4DShader shader = (B4DShader) RenderSystem.getShader();
        if (shader == null) {
            return;
        }

        if (!renderedBuffer.isEmpty()) {
            try(B4DMeshData b4DMeshData = new B4DMeshData()) {
                BufferBuilder.DrawState drawState = renderedBuffer.drawState();
                b4DMeshData.setVertexStride(drawState.format().getVertexSize());
                b4DMeshData.setIndexCount(drawState.indexCount());
                b4DMeshData.setPrimitiveTopology(B4DPrimitiveTopology.fromGLMode(drawState.mode().asGLMode));

                ByteBuffer vertexData = renderedBuffer.vertexBuffer();
                b4DMeshData.setVertexData(MemoryUtil.memAddress(vertexData), vertexData.remaining());

                if(drawState.sequentialIndex()) {
                    b4DMeshData.setIndexType(B4DIndexType.UINT32);

                    IntBuffer indexData = generateSequentialIndices(drawState.mode(), drawState.indexCount());
                    b4DMeshData.setIndexData(MemoryUtil.memAddress(indexData), indexData.remaining() * 4L);
                } else {
                    if (drawState.indexType() == VertexFormat.IndexType.SHORT) {
                        b4DMeshData.setIndexType(B4DIndexType.UINT16);
                    } else if (drawState.indexType() == VertexFormat.IndexType.INT) {
                        b4DMeshData.setIndexType(B4DIndexType.UINT32);
                    } else {
                        return;
                    }

                    ByteBuffer indexData = renderedBuffer.indexBuffer();
                    b4DMeshData.setIndexData(MemoryUtil.memAddress(indexData), indexData.remaining());
                }

                Integer id = Blaze4D.uploadImmediate(b4DMeshData);
                if(id != null) {
                    target.setImmediateData(id);
                }
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
