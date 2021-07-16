package me.hydos.blaze4d.mixin.vertices;

import com.mojang.blaze3d.vertex.BufferBuilder;
import com.mojang.blaze3d.vertex.BufferUploader;
import com.mojang.blaze3d.vertex.VertexFormat;
import com.mojang.datafixers.util.Pair;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import me.hydos.blaze4d.api.vertex.ConsumerCreationInfo;
import me.hydos.rosella.render.vertex.StoredBufferProvider;
import org.joml.Matrix4f;
import org.joml.Vector3f;
import org.lwjgl.system.MemoryUtil;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;

import java.nio.ByteBuffer;

@Mixin(BufferUploader.class)
public class BufferRendererMixin {

    /**
     * @author Blaze4D
     * @reason to draw
     */
    @Overwrite
    public static void end(BufferBuilder bufferBuilder) {
        Matrix4f projMatrix = new Matrix4f(GlobalRenderSystem.projectionMatrix);
        Matrix4f modelViewMatrix = new Matrix4f(GlobalRenderSystem.modelViewMatrix);
        Vector3f chunkOffset = new Vector3f(GlobalRenderSystem.chunkOffset);
        com.mojang.math.Vector3f shaderLightDirections0 = GlobalRenderSystem.shaderLightDirections0.copy();
        com.mojang.math.Vector3f shaderLightDirections1 = GlobalRenderSystem.shaderLightDirections1.copy();

        Pair<BufferBuilder.DrawState, ByteBuffer> drawData = bufferBuilder.popNextBuffer();
        BufferBuilder.DrawState drawState = drawData.getFirst();
        ByteBuffer originalBuffer = drawData.getSecond();

        originalBuffer.clear();

        int vertexCount = drawState.vertexCount();

        if (vertexCount > 0) {
            VertexFormat format = drawState.format();

            ConsumerCreationInfo consumerCreationInfo = new ConsumerCreationInfo(drawState.mode(), format, GlobalRenderSystem.activeShader, GlobalRenderSystem.createTextureArray(), GlobalRenderSystem.currentStateInfo.snapshot(), projMatrix, modelViewMatrix, chunkOffset, shaderLightDirections0, shaderLightDirections1);
            StoredBufferProvider storedBufferProvider = GlobalRenderSystem.getOrCreateBufferProvider(consumerCreationInfo);

            // TODO: figure out a way to accumulate these buffers to a staging buffer throughout the frame.
            // this would get rid of the need to copy the buffer here as well as the need to free the copy.
            ByteBuffer copiedBuffer = MemoryUtil.memAlloc(originalBuffer.limit());
            MemoryUtil.memCopy(originalBuffer, copiedBuffer);
            storedBufferProvider.addBuffer(copiedBuffer, 0, vertexCount, true);
        }
    }

    /**
     * @author Blaze4D
     * @reason to draw
     */
    @Overwrite
    public static void _endInternal(BufferBuilder builder) {
        end(builder);
    }
}
