package graphics.kiln.blaze4d.mixin.render;

import com.mojang.authlib.minecraft.client.MinecraftClient;
import com.mojang.blaze3d.shaders.Uniform;
import com.mojang.blaze3d.systems.RenderSystem;
import com.mojang.blaze3d.vertex.PoseStack;
import com.mojang.blaze3d.vertex.VertexBuffer;
import com.mojang.math.Matrix4f;
import it.unimi.dsi.fastutil.objects.ObjectArrayList;
import it.unimi.dsi.fastutil.objects.ObjectListIterator;
import graphics.kiln.blaze4d.impl.GlobalRenderSystem;
import net.minecraft.client.Minecraft;
import net.minecraft.client.renderer.LevelRenderer;
import net.minecraft.client.renderer.RenderType;
import net.minecraft.client.renderer.ShaderInstance;
import net.minecraft.client.renderer.chunk.ChunkRenderDispatcher;
import net.minecraft.core.BlockPos;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;

@Mixin(LevelRenderer.class)
public class WorldRendererMixin {
    @Shadow private double xTransparentOld;

    @Shadow private double yTransparentOld;

    @Shadow private double zTransparentOld;

/*    *//**
     * This is one of three places where it actually renders a shader.
     * This one is different from the others because most of its setup process is still needed.
     * @author burger
     *//*
    @Overwrite
    public void renderChunkLayer(RenderType renderType, PoseStack modelViewStack, double xTransparent, double yTransparent, double zTransparent, Matrix4f projectionMatrix) {
        RenderSystem.assertOnRenderThread();
        renderType.setupRenderState();
        if (renderType == RenderType.translucent()) {
            Minecraft.getInstance().getProfiler().push("translucent_sort");
            double g = xTransparent - this.xTransparentOld;
            double h = yTransparent - this.yTransparentOld;
            double i = zTransparent - this.zTransparentOld;
            if (g * g + h * h + i * i > 1.0) {
                this.xTransparentOld = xTransparent;
                this.yTransparentOld = yTransparent;
                this.zTransparentOld = zTransparent;
                int j = 0;

                for(LevelRenderer.RenderChunkInfo renderChunkInfo : this.renderChunks) {
                    if (j < 15 && renderChunkInfo.chunk.resortTransparency(renderType, this.chunkRenderDispatcher)) {
                        ++j;
                    }
                }
            }

            Minecraft.getInstance().getProfiler().pop();
        }

        Minecraft.getInstance().getProfiler().push("filterempty");
        Minecraft.getInstance().getProfiler().popPush(() -> "render_" + renderType);
        boolean bl = renderType != RenderType.translucent();
        ObjectListIterator<LevelRenderer.RenderChunkInfo> objectListIterator = this.renderChunks.listIterator(bl ? 0 : this.renderChunks.size());

        ShaderInstance shader = RenderSystem.getShader();

        GlobalRenderSystem.updateUniforms(shader, modelViewStack.last().pose(), projectionMatrix);

        Uniform chunkOffset = shader.CHUNK_OFFSET;

        while(true) {
            if (bl) {
                if (!objectListIterator.hasNext()) {
                    break;
                }
            } else if (!objectListIterator.hasPrevious()) {
                break;
            }

            LevelRenderer.RenderChunkInfo renderChunkInfo2 = bl ? objectListIterator.next() : objectListIterator.previous();
            ChunkRenderDispatcher.RenderChunk renderChunk = renderChunkInfo2.chunk;
            if (!renderChunk.getCompiledChunk().isEmpty(renderType)) {
                VertexBuffer vertexBuffer = renderChunk.getBuffer(renderType);
                BlockPos blockPos = renderChunk.getOrigin();
                if (chunkOffset != null) {
                    chunkOffset.set((float)((double)blockPos.getX() - xTransparent), (float)((double)blockPos.getY() - yTransparent), (float)((double)blockPos.getZ() - zTransparent));
                }

                vertexBuffer.drawChunkLayer();
            }
        }

        shader.clear();
        Minecraft.getInstance().getProfiler().pop();
        renderType.clearRenderState();
    }*/
}
