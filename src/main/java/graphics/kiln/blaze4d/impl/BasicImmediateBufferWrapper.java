package graphics.kiln.blaze4d.impl;

import com.mojang.blaze3d.systems.RenderSystem;
import com.mojang.blaze3d.vertex.BufferBuilder;
import com.mojang.datafixers.util.Pair;
import graphics.kiln.blaze4d.api.render.ImmediateBufferWrapper;
import graphics.kiln.rosella.memory.ManagedBuffer;
import graphics.kiln.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import me.hydos.blaze4d.api.util.ConversionUtils;
import org.lwjgl.system.MemoryUtil;

import java.nio.ByteBuffer;

public class BasicImmediateBufferWrapper implements ImmediateBufferWrapper {

    @Override
    public void render(BufferBuilder builder) {
        Pair<BufferBuilder.DrawState, ByteBuffer> drawData = builder.popNextBuffer();
        BufferBuilder.DrawState drawState = drawData.getFirst();
        ByteBuffer originalBuffer = drawData.getSecond();

        originalBuffer.clear();

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
                    ConversionUtils.FORMAT_CONVERSION_MAP.get(drawState.format().getElements()),
                    GlobalRenderSystem.currentStateInfo.snapshot(),
                    GlobalRenderSystem.getCurrentTextureMap(),
                    GlobalRenderSystem.getShaderUbo(RenderSystem.getShader()),
                    Blaze4D.rosella
            );
        }
    }
}
