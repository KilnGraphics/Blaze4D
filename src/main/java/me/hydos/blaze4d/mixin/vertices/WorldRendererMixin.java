package me.hydos.blaze4d.mixin.vertices;

import com.mojang.blaze3d.shaders.Uniform;
import com.mojang.blaze3d.systems.RenderSystem;
import com.mojang.blaze3d.vertex.BufferUploader;
import com.mojang.blaze3d.vertex.PoseStack;
import com.mojang.blaze3d.vertex.VertexBuffer;
import com.mojang.blaze3d.vertex.VertexFormat;
import com.mojang.math.Matrix4f;
import com.mojang.math.Vector3f;
import it.unimi.dsi.fastutil.objects.ObjectArrayList;
import it.unimi.dsi.fastutil.objects.ObjectListIterator;
import me.hydos.blaze4d.api.GlobalRenderSystem;
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

    @Shadow @Final private Minecraft minecraft;

    @Shadow @Final private ObjectArrayList<LevelRenderer.RenderChunkInfo> renderChunks;

    @Shadow private ChunkRenderDispatcher chunkRenderDispatcher;

    /**
     * This is one of three places where it actually renders a shader.
     * This one is different from the others because most of its setup process is still needed.
     * @author burger
     */
    @Overwrite
    private void renderChunkLayer(RenderType renderType, PoseStack poseStack, double xTransparent, double yTransparent, double zTransparent, Matrix4f matrix4f) {

        RenderSystem.assertThread(RenderSystem::isOnRenderThread);
        renderType.setupRenderState();
        if (renderType == RenderType.translucent()) {
            this.minecraft.getProfiler().push("translucent_sort");
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

            this.minecraft.getProfiler().pop();
        }

        this.minecraft.getProfiler().push("filterempty");
        this.minecraft.getProfiler().popPush(() -> "render_" + renderType);
        boolean bl = renderType != RenderType.translucent();
        ObjectListIterator<LevelRenderer.RenderChunkInfo> objectListIterator = this.renderChunks.listIterator(bl ? 0 : this.renderChunks.size());

        GlobalRenderSystem.updateUniforms();

        Uniform chunkOffset = RenderSystem.getShader().CHUNK_OFFSET;

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
                    chunkOffset.upload();
                }

                vertexBuffer.drawChunkLayer();
            }
        }

        if (chunkOffset != null) {
            chunkOffset.set(Vector3f.ZERO);
        }

        RenderSystem.getShader().clear();
        this.minecraft.getProfiler().pop();
        renderType.clearRenderState();
    }
}
