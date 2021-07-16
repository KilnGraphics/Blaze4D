package me.hydos.blaze4d.mixin.vertices;

import com.mojang.blaze3d.vertex.BufferBuilder;
import com.mojang.blaze3d.vertex.VertexBuffer;
import com.mojang.blaze3d.vertex.VertexFormat;
import com.mojang.datafixers.util.Pair;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import me.hydos.blaze4d.api.util.ConversionUtils;
import me.hydos.blaze4d.api.vertex.ConsumerCreationInfo;
import me.hydos.rosella.render.vertex.StoredBufferProvider;
import net.minecraft.client.renderer.ShaderInstance;
import org.joml.Matrix4f;
import org.joml.Vector3f;
import org.lwjgl.system.MemoryUtil;
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
    private ByteBuffer buffer;
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

            ByteBuffer copiedBuffer = MemoryUtil.memAlloc(originalBuffer.limit());
            MemoryUtil.memCopy(originalBuffer, copiedBuffer);
            if (this.buffer != null) {
                MemoryUtil.memFree(this.buffer);
            }
            this.buffer = copiedBuffer;
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
            VertexFormat format = drawState.format();

            ConsumerCreationInfo consumerCreationInfo = new ConsumerCreationInfo(drawState.mode(), format, GlobalRenderSystem.activeShader, GlobalRenderSystem.createTextureArray(), GlobalRenderSystem.currentStateInfo.snapshot(), projMatrix, modelViewMatrix, chunkOffset, shaderLightDirections0, shaderLightDirections1);
            StoredBufferProvider storedBufferProvider = GlobalRenderSystem.getOrCreateBufferProvider(consumerCreationInfo);

            storedBufferProvider.addBuffer(this.buffer, 0, vertexCount, false);
        }
    }

    @Inject(method = "close", at = @At("HEAD"), cancellable = true)
    private void close(CallbackInfo ci) {
        MemoryUtil.memFree(buffer);
        ci.cancel();
    }
}
