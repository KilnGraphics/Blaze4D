package me.hydos.blaze4d.mixin.vertices;

import com.mojang.blaze3d.vertex.BufferBuilder;
import com.mojang.blaze3d.vertex.BufferUploader;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import org.joml.Matrix4f;
import org.joml.Vector3f;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

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

        GlobalRenderSystem.drawVertices(projMatrix, modelViewMatrix, chunkOffset, shaderLightDirections0, shaderLightDirections1, bufferBuilder.popNextBuffer());
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
