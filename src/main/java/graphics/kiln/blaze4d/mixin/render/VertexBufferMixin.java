package graphics.kiln.blaze4d.mixin.render;

import com.mojang.blaze3d.systems.RenderSystem;
import com.mojang.blaze3d.vertex.*;
import com.mojang.datafixers.util.Pair;
import graphics.kiln.blaze4d.Blaze4D;
import graphics.kiln.blaze4d.api.B4DShader;
import graphics.kiln.blaze4d.mixin.shader.ShaderMixin;
import net.minecraft.client.renderer.ShaderInstance;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.ModifyVariable;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.nio.ByteBuffer;
import java.util.Objects;

/**
 * Turns out, Minecraft uses this class for world rendering. when a part of the world is to be rendered, the buffer will be cleared and replaced with just the sky. this will then be uploaded to a {@link VertexBuffer} and then cleared again for the rest of the game to render.
 */
@Mixin(VertexBuffer.class)
public class VertexBufferMixin {

    private long staticMeshId = 0L;

    /**
     * @author Blaze4D
     * @reason Allow for uploading Vertex Buffers
     */
    @Inject(method="upload", at = @At("HEAD"))
    private void uploadBuffer(BufferBuilder.RenderedBuffer renderedBuffer) {
        if (this.staticMeshId != 0L) {
            Blaze4D.core.destroyStaticMesh(this.staticMeshId);
            this.staticMeshId = 0;
        }

        BufferBuilder.DrawState drawState = renderedBuffer.drawState();
    }
//
//    /**
//     * @author Blaze4D
//     * @reason Allows rendering things such as the sky.
//     */
//    @Overwrite
//    public void _drawWithShader(com.mojang.math.Matrix4f mcModelViewMatrix, com.mojang.math.Matrix4f mcProjectionMatrix, ShaderInstance shader) {
//        GlobalRenderSystem.updateUniforms(shader, mcModelViewMatrix, mcProjectionMatrix);
//        callWrapperRender(shader);
//    }
//
//
//    @Unique
//    private void callWrapperRender(ShaderInstance mcShader) {
//        RawShaderProgram rawProgram = GlobalRenderSystem.SHADER_PROGRAM_MAP.get(mcShader.getId());
//        ShaderProgram rosellaShaderProgram = Blaze4D.rosella.common.shaderManager.getOrCreateShader(rawProgram);
//        wrapper.render(rosellaShaderProgram, GlobalRenderSystem.getShaderUbo(mcShader));
//    }
//
    @Inject(method = "close", at = @At("HEAD"), cancellable = true)
    private void close(CallbackInfo ci) {
        if (this.staticMeshId != 0L) {
            Blaze4D.core.destroyStaticMesh(this.staticMeshId);
        }
    }
}
