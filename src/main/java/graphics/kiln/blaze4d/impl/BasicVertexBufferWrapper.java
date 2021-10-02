package graphics.kiln.blaze4d.impl;

import com.mojang.blaze3d.vertex.BufferBuilder;
import com.mojang.datafixers.util.Pair;
import graphics.kiln.blaze4d.api.render.VertexBufferWrapper;
import graphics.kiln.rosella.Rosella;
import graphics.kiln.rosella.memory.BufferInfo;
import graphics.kiln.rosella.memory.ManagedBuffer;
import graphics.kiln.rosella.render.Topology;
import graphics.kiln.rosella.render.info.RenderInfo;
import graphics.kiln.rosella.render.shader.ShaderProgram;
import graphics.kiln.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import me.hydos.blaze4d.api.util.ConversionUtils;

import java.nio.ByteBuffer;

public class BasicVertexBufferWrapper implements VertexBufferWrapper {

    private final Rosella rosella;
    private RenderInfo currentRenderInfo;
    private BufferBuilder.DrawState drawState;
    private Topology convertedTopology;
    private graphics.kiln.rosella.render.vertex.VertexFormat convertedVertexFormat;

    public BasicVertexBufferWrapper(Rosella rosella) {
        this.rosella = rosella;
    }

    @Override
    public void create(BufferBuilder bufferBuilder) {
        Pair<BufferBuilder.DrawState, ByteBuffer> drawData = bufferBuilder.popNextBuffer();
        // We have to manipulate some stuff with the ByteBuffer before we store it
        BufferBuilder.DrawState providedDrawState = drawData.getFirst();
        ByteBuffer providedBuffer = drawData.getSecond();

        if (providedDrawState.vertexCount() > 0 && providedDrawState.indexCount() > 0) {
            if (!providedDrawState.indexOnly()) {
                providedBuffer.limit(providedDrawState.vertexBufferSize());
                BufferInfo vertexBuffer = Blaze4D.rosella.bufferManager.createVertexBuffer(new ManagedBuffer<>(providedBuffer, false));

                GlobalRenderSystem.MinecraftIndexBuffer mcIndexBuffer = GlobalRenderSystem.createIndices(providedDrawState.mode(), providedDrawState.indexCount());
                BufferInfo indexBuffer = Blaze4D.rosella.bufferManager.createIndexBuffer(mcIndexBuffer.rawBuffer());

                currentRenderInfo = new RenderInfo(vertexBuffer, indexBuffer, mcIndexBuffer.newIndexCount());
                drawState = providedDrawState;
                convertedTopology = ConversionUtils.mcDrawModeToRosellaTopology(mcIndexBuffer.newMode());
                convertedVertexFormat = ConversionUtils.FORMAT_CONVERSION_MAP.get(providedDrawState.format().getElements());
            }
        }

        providedBuffer.limit(providedDrawState.bufferSize());
        providedBuffer.position(0);
    }

    @Override
    public void render(ShaderProgram shaderProgram, ByteBuffer uboData) {
        if (currentRenderInfo != null && drawState != null) {
            GlobalRenderSystem.uploadPreCreatedObject(
                    currentRenderInfo,
                    shaderProgram,
                    convertedTopology,
                    GlobalRenderSystem.DEFAULT_POLYGON_MODE,
                    convertedVertexFormat,
                    GlobalRenderSystem.currentStateInfo.snapshot(),
                    GlobalRenderSystem.getCurrentTextureMap(),
                    uboData,
                    Blaze4D.rosella
            );
        }
    }

    @Override
    public void clean() {
        if (currentRenderInfo != null) currentRenderInfo.free(Blaze4D.rosella.common.device, Blaze4D.rosella.common.memory);
    }
}
