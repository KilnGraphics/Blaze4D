package me.hydos.blaze4d.mixin.vertices;

import com.mojang.blaze3d.vertex.BufferBuilder;
import com.mojang.blaze3d.vertex.VertexBuffer;
import com.mojang.datafixers.util.Pair;
import it.unimi.dsi.fastutil.objects.ObjectIntPair;
import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import me.hydos.blaze4d.api.util.ConversionUtils;
import me.hydos.rosella.memory.BufferInfo;
import me.hydos.rosella.memory.ManagedBuffer;
import me.hydos.rosella.render.info.RenderInfo;
import net.minecraft.client.renderer.ShaderInstance;
import org.joml.Matrix4f;
import org.joml.Vector3f;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.nio.ByteBuffer;

/**
 * Turns out, Minecraft uses this class for world rendering. when a part of the world is to be rendered, the buffer will be cleared and replaced with just the sky. this will then be uploaded to a {@link VertexBuffer} and then cleared again for the rest of the game to render.
 * TODO OPT: redo this whole class and allow for custom vertex buffers outside of the main giant one in rosella.
 */
@Mixin(VertexBuffer.class)
public class VertexBufferMixin {

    @Unique
    private RenderInfo currentRenderInfo;
    @Unique
    private BufferBuilder.DrawState drawState;

    /**
     * @author Blaze4D
     * @reason To render
     */
    @Overwrite
    private void upload_(BufferBuilder bufferBuilder) {
        Pair<BufferBuilder.DrawState, ByteBuffer> drawData = bufferBuilder.popNextBuffer();
        // We have to manipulate some stuff with the ByteBuffer before we store it
        this.drawState = drawData.getFirst();
        ByteBuffer originalBuffer = drawData.getSecond();

        if (!drawState.indexOnly()) {
            originalBuffer.limit(drawState.vertexBufferSize());
            BufferInfo vertexBuffer = Blaze4D.rosella.bufferManager.createVertexBuffer(new ManagedBuffer<>(originalBuffer, false));

            ObjectIntPair<ManagedBuffer<ByteBuffer>> indexBufferSourcePair = GlobalRenderSystem.createIndices(drawState.mode(), drawState.vertexCount());
            int indexCount = indexBufferSourcePair.valueInt();
            BufferInfo indexBuffer = Blaze4D.rosella.bufferManager.createIndexBuffer(indexBufferSourcePair.key());

            currentRenderInfo = new RenderInfo(vertexBuffer, indexBuffer, indexCount);
        }

        originalBuffer.limit(drawState.bufferSize());
        originalBuffer.position(0);
    }

    /**
     * @author Blaze4D
     * @reason To render the sky
     */
    @Overwrite
    public void _drawWithShader(com.mojang.math.Matrix4f mcModelViewMatrix, com.mojang.math.Matrix4f mcProjectionMatrix, ShaderInstance shader) {
        Matrix4f projMatrix = ConversionUtils.mcToJomlProjectionMatrix(mcProjectionMatrix);
        Matrix4f modelViewMatrix = ConversionUtils.mcToJomlMatrix(mcModelViewMatrix);
        Vector3f chunkOffset = new Vector3f(GlobalRenderSystem.chunkOffset);
        com.mojang.math.Vector3f shaderLightDirections0 = GlobalRenderSystem.shaderLightDirections0.copy();
        com.mojang.math.Vector3f shaderLightDirections1 = GlobalRenderSystem.shaderLightDirections1.copy();

        addBufferToRosella(projMatrix, modelViewMatrix, chunkOffset, shaderLightDirections0, shaderLightDirections1);
    }

    /**
     * @author Blaze4D
     * @reason To render the world
     */
    @Overwrite
    public void drawChunkLayer() {
        Matrix4f projMatrix = new Matrix4f(GlobalRenderSystem.tmpProjectionMatrix);
        Matrix4f modelViewMatrix = new Matrix4f(GlobalRenderSystem.tmpModelViewMatrix);
        Vector3f chunkOffset = new Vector3f(GlobalRenderSystem.chunkOffset);
        com.mojang.math.Vector3f shaderLightDirections0 = GlobalRenderSystem.shaderLightDirections0.copy();
        com.mojang.math.Vector3f shaderLightDirections1 = GlobalRenderSystem.shaderLightDirections1.copy();

        addBufferToRosella(projMatrix, modelViewMatrix, chunkOffset, shaderLightDirections0, shaderLightDirections1);
    }

    @Unique
    private void addBufferToRosella(Matrix4f projMatrix, Matrix4f modelViewMatrix, Vector3f chunkOffset, com.mojang.math.Vector3f shaderLightDirections0, com.mojang.math.Vector3f shaderLightDirections1) {
        int vertexCount = drawState.vertexCount();

        if (vertexCount > 0) {
            GlobalRenderSystem.uploadPreCreatedObject(
                    currentRenderInfo,
                    ConversionUtils.FORMAT_CONVERSION_MAP.get(drawState.format().getElements()),
                    GlobalRenderSystem.activeShader,
                    GlobalRenderSystem.createTextureArray(),
                    GlobalRenderSystem.currentStateInfo.snapshot(),
                    projMatrix,
                    modelViewMatrix,
                    chunkOffset,
                    shaderLightDirections0,
                    shaderLightDirections1,
                    drawState.format(),
                    drawState.mode(),
                    Blaze4D.rosella
            );
        }
    }

    @Inject(method = "close", at = @At("HEAD"), cancellable = true)
    private void close(CallbackInfo ci) {
        currentRenderInfo.free(Blaze4D.rosella.common.device, Blaze4D.rosella.common.memory);
        ci.cancel();
    }
}
