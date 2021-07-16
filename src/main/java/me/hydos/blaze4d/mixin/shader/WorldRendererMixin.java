package me.hydos.blaze4d.mixin.shader;

import com.mojang.blaze3d.shaders.Uniform;
import com.mojang.blaze3d.vertex.PoseStack;
import com.mojang.blaze3d.vertex.VertexBuffer;
import com.mojang.blaze3d.vertex.VertexFormat;
import com.mojang.math.Matrix4f;
import it.unimi.dsi.fastutil.objects.ObjectListIterator;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import me.hydos.blaze4d.api.shader.MinecraftUbo;
import me.hydos.blaze4d.api.util.ConversionUtils;
import net.minecraft.client.renderer.LevelRenderer;
import net.minecraft.client.renderer.RenderType;
import net.minecraft.client.renderer.ShaderInstance;
import net.minecraft.client.renderer.chunk.ChunkRenderDispatcher;
import net.minecraft.core.BlockPos;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.LocalCapture;

@Mixin(value = LevelRenderer.class, priority = 1001)
public class WorldRendererMixin {

    @Inject(method = "renderChunkLayer", at = @At(value = "INVOKE", target = "Lcom/mojang/blaze3d/shaders/Uniform;set(FFF)V"), locals = LocalCapture.CAPTURE_FAILSOFT)
    private void redirectChunkOffset(RenderType renderLayer, PoseStack matrices, double x, double y, double z, Matrix4f matrix4f, CallbackInfo ci, boolean bl, ObjectListIterator<?> objectListIterator, VertexFormat vertexFormat, ShaderInstance shader, Uniform glUniform, boolean bl2, LevelRenderer.RenderChunkInfo chunkInfo2, ChunkRenderDispatcher.RenderChunk builtChunk, VertexBuffer vertexBuffer, BlockPos blockPos) {
        GlobalRenderSystem.chunkOffset.set((double) blockPos.getX() - x, (double) blockPos.getY() - y, (double) blockPos.getZ() - z);
    }

    @Inject(method = "renderChunkLayer", at = @At(value = "INVOKE", target = "Lcom/mojang/blaze3d/shaders/Uniform;set(Lcom/mojang/math/Matrix4f;)V", ordinal = 0))
    private void redirectModelViewMatrix(RenderType renderLayer, PoseStack matrices, double d, double e, double f, Matrix4f modelViewMatrix, CallbackInfo ci) {
        GlobalRenderSystem.tmpModelViewMatrix.set(ConversionUtils.mcToJomlMatrix(matrices.last().pose()));
    }

    @Inject(method = "renderChunkLayer", at = @At(value = "INVOKE", target = "Lcom/mojang/blaze3d/shaders/Uniform;set(Lcom/mojang/math/Matrix4f;)V", ordinal = 1))
    private void redirectProjectionMatrix(RenderType renderLayer, PoseStack matrices, double d, double e, double f, Matrix4f projectionMatrix, CallbackInfo ci) {
        GlobalRenderSystem.tmpProjectionMatrix.set(ConversionUtils.mcToJomlProjectionMatrix(projectionMatrix));
    }
}
