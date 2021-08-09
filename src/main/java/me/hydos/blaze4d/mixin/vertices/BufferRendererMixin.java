package me.hydos.blaze4d.mixin.vertices;

import com.mojang.blaze3d.vertex.BufferBuilder;
import com.mojang.blaze3d.vertex.BufferUploader;
import com.mojang.datafixers.util.Pair;
import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import me.hydos.blaze4d.api.util.ConversionUtils;
import me.hydos.rosella.memory.ManagedBuffer;
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
        Pair<BufferBuilder.DrawState, ByteBuffer> drawData = bufferBuilder.popNextBuffer();
        BufferBuilder.DrawState drawState = drawData.getFirst();
        ByteBuffer originalBuffer = drawData.getSecond();

        originalBuffer.clear();

        // TODO: why were these format checks here? (ported from old code) drawState.format() != com.mojang.blaze3d.vertex.DefaultVertexFormat.BLIT_SCREEN && drawState.format() != com.mojang.blaze3d.vertex.DefaultVertexFormat.POSITION
        if (drawState.vertexCount() > 0 && drawState.indexCount() > 0) {
            ByteBuffer copiedBuffer = MemoryUtil.memAlloc(drawState.vertexBufferSize());
            copiedBuffer.put(0, originalBuffer, 0, drawState.vertexBufferSize());

            GlobalRenderSystem.MinecraftIndexBuffer mcIndexBuffer = GlobalRenderSystem.createIndices(drawState.mode(), drawState.indexCount());

            GlobalRenderSystem.updateUniforms();

            GlobalRenderSystem.uploadAsyncCreatableObject(
                    new ManagedBuffer<>(copiedBuffer, true),
                    mcIndexBuffer.rawBuffer(),
                    mcIndexBuffer.newIndexCount(),
                    GlobalRenderSystem.activeShader,
                    ConversionUtils.mcDrawModeToRosellaTopology(mcIndexBuffer.newMode()),
                    GlobalRenderSystem.DEFAULT_POLYGON_MODE,
                    ConversionUtils.FORMAT_CONVERSION_MAP.get(drawState.format().getElements()),
                    GlobalRenderSystem.currentStateInfo.snapshot(),
                    GlobalRenderSystem.getCurrentTextureMap(),
                    Blaze4D.rosella
            );
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
