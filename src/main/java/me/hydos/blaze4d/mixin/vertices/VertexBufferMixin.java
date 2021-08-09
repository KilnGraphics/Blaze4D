package me.hydos.blaze4d.mixin.vertices;

import com.mojang.blaze3d.systems.RenderSystem;
import com.mojang.blaze3d.vertex.BufferBuilder;
import com.mojang.blaze3d.vertex.VertexBuffer;
import com.mojang.datafixers.util.Pair;
import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import me.hydos.blaze4d.api.util.ConversionUtils;
import me.hydos.rosella.memory.BufferInfo;
import me.hydos.rosella.memory.ManagedBuffer;
import me.hydos.rosella.render.Topology;
import me.hydos.rosella.render.info.RenderInfo;
import me.hydos.rosella.render.shader.RawShaderProgram;
import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.scene.object.impl.SimpleObjectManager;
import net.minecraft.client.renderer.ShaderInstance;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.nio.ByteBuffer;

/**
 * Turns out, Minecraft uses this class for world rendering. when a part of the world is to be rendered, the buffer will be cleared and replaced with just the sky. this will then be uploaded to a {@link VertexBuffer} and then cleared again for the rest of the game to render.
 */
@Mixin(VertexBuffer.class)
public class VertexBufferMixin {
    @Unique
    private RenderInfo currentRenderInfo;
    @Unique
    private BufferBuilder.DrawState drawState;
    @Unique
    private Topology convertedTopology;
    @Unique
    private me.hydos.rosella.render.vertex.VertexFormat convertedVertexFormat;

    /**
     * @author Blaze4D
     * @reason To render
     */
    @Overwrite
    private void upload_(BufferBuilder bufferBuilder) {
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

    /**
     * @author Blaze4D
     * @reason To render the sky
     */
    @Overwrite
    public void _drawWithShader(com.mojang.math.Matrix4f mcModelViewMatrix, com.mojang.math.Matrix4f mcProjectionMatrix, ShaderInstance shader) {
        GlobalRenderSystem.updateUniforms(shader, mcModelViewMatrix, mcProjectionMatrix);
        addBufferToRosella(shader);
    }

    /**
     * @author Blaze4D
     * @reason To render the world
     */
    @Overwrite
    public void drawChunkLayer() {
        addBufferToRosella(GlobalRenderSystem.activeShader, GlobalRenderSystem.getShaderUbo(RenderSystem.getShader()));
    }

    @Unique
    private void addBufferToRosella(ShaderInstance mcShader) {
        RawShaderProgram rawProgram = GlobalRenderSystem.SHADER_PROGRAM_MAP.get(mcShader.getId());
        ShaderProgram rosellaShaderProgram = ((SimpleObjectManager) Blaze4D.rosella.objectManager).shaderManager.getOrCreateShader(rawProgram);
        addBufferToRosella(rosellaShaderProgram, GlobalRenderSystem.getShaderUbo(mcShader));
    }

    @Unique
    private void addBufferToRosella(ShaderProgram rosellaShaderProgram, ByteBuffer rawUboData) {
        if (currentRenderInfo != null && drawState != null) {
            GlobalRenderSystem.uploadPreCreatedObject(
                    currentRenderInfo,
                    rosellaShaderProgram,
                    convertedTopology,
                    GlobalRenderSystem.DEFAULT_POLYGON_MODE,
                    convertedVertexFormat,
                    GlobalRenderSystem.currentStateInfo.snapshot(),
                    GlobalRenderSystem.getCurrentTextureMap(),
                    rawUboData,
                    Blaze4D.rosella
            );
        }
    }

    @Inject(method = "close", at = @At("HEAD"), cancellable = true)
    private void close(CallbackInfo ci) {
        if (currentRenderInfo != null) currentRenderInfo.free(Blaze4D.rosella.common.device, Blaze4D.rosella.common.memory);
        ci.cancel();
    }
}
