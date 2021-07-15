package me.hydos.blaze4d.mixin.shader;

import it.unimi.dsi.fastutil.objects.ObjectListIterator;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import me.hydos.blaze4d.api.shader.MinecraftUbo;
import me.hydos.blaze4d.api.util.ConversionUtils;
import net.minecraft.client.gl.GlUniform;
import net.minecraft.client.gl.VertexBuffer;
import net.minecraft.client.render.RenderLayer;
import net.minecraft.client.render.Shader;
import net.minecraft.client.render.VertexFormat;
import net.minecraft.client.render.WorldRenderer;
import net.minecraft.client.render.chunk.ChunkBuilder;
import net.minecraft.client.util.math.MatrixStack;
import net.minecraft.util.math.BlockPos;
import net.minecraft.util.math.Matrix4f;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.LocalCapture;

@Mixin(value = WorldRenderer.class, priority = 1001)
public class WorldRendererMixin {

    @Inject(method = "renderLayer", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/gl/GlUniform;set(FFF)V"), require = 0, locals = LocalCapture.CAPTURE_FAILSOFT)
    private void redirectChunkOffset(RenderLayer renderLayer, MatrixStack matrices, double x, double y, double z, Matrix4f matrix4f, CallbackInfo ci, boolean bl, ObjectListIterator<?> objectListIterator, VertexFormat vertexFormat, Shader shader, GlUniform glUniform, boolean bl2, WorldRenderer.ChunkInfo chunkInfo2, ChunkBuilder.BuiltChunk builtChunk, VertexBuffer vertexBuffer, BlockPos blockPos) {
        GlobalRenderSystem.chunkOffset.set((double) blockPos.getX() - x, (double) blockPos.getY() - y, (double) blockPos.getZ() - z);
    }

    @Inject(method = "renderLayer", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/gl/GlUniform;set(Lnet/minecraft/util/math/Matrix4f;)V", ordinal = 0), require = 0)
    private void redirectModelViewMatrix(RenderLayer renderLayer, MatrixStack matrices, double d, double e, double f, Matrix4f modelViewMatrix, CallbackInfo ci) {
        GlobalRenderSystem.modelViewMatrix.set(ConversionUtils.mcToJomlMatrix(matrices.peek().getModel()));
    }

    @Inject(method = "renderLayer", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/gl/GlUniform;set(Lnet/minecraft/util/math/Matrix4f;)V", ordinal = 1), require = 0)
    private void redirectProjectionMatrix(RenderLayer renderLayer, MatrixStack matrices, double d, double e, double f, Matrix4f projectionMatrix, CallbackInfo ci) {
        GlobalRenderSystem.projectionMatrix.set(ConversionUtils.mcToJomlProjectionMatrix(projectionMatrix));
    }
}
