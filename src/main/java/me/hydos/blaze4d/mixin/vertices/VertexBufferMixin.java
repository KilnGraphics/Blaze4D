package me.hydos.blaze4d.mixin.vertices;

import com.mojang.datafixers.util.Pair;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import me.hydos.blaze4d.api.shader.MinecraftUbo;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.gl.VertexBuffer;
import net.minecraft.client.render.BufferBuilder;
import net.minecraft.client.render.Camera;
import net.minecraft.client.render.Shader;
import net.minecraft.util.math.Vec3f;
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

    private Pair<BufferBuilder.DrawArrayParameters, ByteBuffer> drawData;

    /**
     * @author Blaze4D
     * @reason To render
     */
    @Overwrite
    private void uploadInternal(BufferBuilder bufferBuilder) {
        this.drawData = bufferBuilder.popData();
    }

    /**
     * @author Blaze4D
     * @reason To render the sky
     */
    @Overwrite
    public void innerSetShader(net.minecraft.util.math.Matrix4f mcModelViewMatrix, net.minecraft.util.math.Matrix4f mcProjectionMatrix, Shader shader) {
        Matrix4f projMatrix = new Matrix4f(GlobalRenderSystem.projectionMatrix);
        Matrix4f modelViewMatrix = MinecraftUbo.toJoml(mcModelViewMatrix);
        Vector3f chunkOffset = new Vector3f(GlobalRenderSystem.chunkOffset);
        Vec3f shaderLightDirections0 = GlobalRenderSystem.shaderLightDirections0.copy();
        Vec3f shaderLightDirections1 = GlobalRenderSystem.shaderLightDirections1.copy();
        GlobalRenderSystem.drawVertices(projMatrix, modelViewMatrix, chunkOffset, shaderLightDirections0, shaderLightDirections1, drawData);
    }

    /**
     * @author Blaze4D
     * @reason To render the world
     */
    @Overwrite
    public void drawVertices() {
        Matrix4f projMatrix = new Matrix4f(GlobalRenderSystem.projectionMatrix);
        Matrix4f modelViewMatrix = new Matrix4f(GlobalRenderSystem.modelViewMatrix);
        Vector3f chunkOffset = new Vector3f(GlobalRenderSystem.chunkOffset);
        Vec3f shaderLightDirections0 = GlobalRenderSystem.shaderLightDirections0.copy();
        Vec3f shaderLightDirections1 = GlobalRenderSystem.shaderLightDirections1.copy();
        GlobalRenderSystem.drawVertices(projMatrix, modelViewMatrix, chunkOffset, shaderLightDirections0, shaderLightDirections1, drawData);
    }
}
