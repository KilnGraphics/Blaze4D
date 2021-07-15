package me.hydos.blaze4d.mixin.vertices;

import com.mojang.blaze3d.vertex.BufferBuilder;
import com.mojang.blaze3d.vertex.VertexBuffer;
import com.mojang.datafixers.util.Pair;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import me.hydos.blaze4d.api.shader.MinecraftUbo;
import me.hydos.blaze4d.api.util.ConversionUtils;
import net.minecraft.client.renderer.ShaderInstance;
import org.joml.Matrix4f;
import org.joml.Vector3f;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;

import java.nio.ByteBuffer;

/**
 * Turns out, Minecraft uses this class for world rendering. when a part of the world is to be rendered, the buffer will be cleared and replaced with just the sky. this will then be uploaded to a {@link VertexBuffer} and then cleared again for the rest of the game to render.
 */
@Mixin(VertexBuffer.class)
public class VertexBufferMixin {

    private Pair<BufferBuilder.DrawState, ByteBuffer> drawData;

    /**
     * @author Blaze4D
     * @reason To render
     */
    @Overwrite
    private void upload_(BufferBuilder bufferBuilder) {
        // TODO OPT: upload to rosella once but use a flag to make sure it's not cleared. it can then be referenced with an int to redraw.
        this.drawData = bufferBuilder.popNextBuffer();
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
        GlobalRenderSystem.drawVertices(projMatrix, modelViewMatrix, chunkOffset, shaderLightDirections0, shaderLightDirections1, drawData);
    }

    /**
     * @author Blaze4D
     * @reason To render the world
     */
    @Overwrite
    public void drawChunkLayer() {
        Matrix4f projMatrix = new Matrix4f(GlobalRenderSystem.projectionMatrix);
        Matrix4f modelViewMatrix = new Matrix4f(GlobalRenderSystem.modelViewMatrix);
        Vector3f chunkOffset = new Vector3f(GlobalRenderSystem.chunkOffset);
        com.mojang.math.Vector3f shaderLightDirections0 = GlobalRenderSystem.shaderLightDirections0.copy();
        com.mojang.math.Vector3f shaderLightDirections1 = GlobalRenderSystem.shaderLightDirections1.copy();
        GlobalRenderSystem.drawVertices(projMatrix, modelViewMatrix, chunkOffset, shaderLightDirections0, shaderLightDirections1, drawData);
    }
}
