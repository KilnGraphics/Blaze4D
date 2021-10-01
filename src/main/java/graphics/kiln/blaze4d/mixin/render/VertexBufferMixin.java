package graphics.kiln.blaze4d.mixin.render;

import com.mojang.blaze3d.systems.RenderSystem;
import com.mojang.blaze3d.vertex.BufferBuilder;
import com.mojang.blaze3d.vertex.VertexBuffer;
import graphics.kiln.blaze4d.impl.BasicVertexBufferWrapper;
import graphics.kiln.rosella.render.shader.RawShaderProgram;
import graphics.kiln.rosella.render.shader.ShaderProgram;
import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import net.minecraft.client.renderer.ShaderInstance;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Turns out, Minecraft uses this class for world rendering. when a part of the world is to be rendered, the buffer will be cleared and replaced with just the sky. this will then be uploaded to a {@link VertexBuffer} and then cleared again for the rest of the game to render.
 */
@Mixin(VertexBuffer.class)
public class VertexBufferMixin {

    private final BasicVertexBufferWrapper wrapper = new BasicVertexBufferWrapper(Blaze4D.rosella);

    /**
     * @author Blaze4D
     * @reason Allow for uploading Vertex Buffers
     */
    @Overwrite
    private void upload_(BufferBuilder bufferBuilder) {
        wrapper.create(bufferBuilder);
    }

    /**
     * @author Blaze4D
     * @reason Allows rendering things such as the sky.
     */
    @Overwrite
    public void _drawWithShader(com.mojang.math.Matrix4f mcModelViewMatrix, com.mojang.math.Matrix4f mcProjectionMatrix, ShaderInstance shader) {
        GlobalRenderSystem.updateUniforms(shader, mcModelViewMatrix, mcProjectionMatrix);
        callWrapperRender(shader);
    }

    /**
     * @author Blaze4D
     * @reason Allows rendering things such as Chunks within a World.
     */
    @Overwrite
    public void drawChunkLayer() {
        wrapper.render(GlobalRenderSystem.activeShader, GlobalRenderSystem.getShaderUbo(RenderSystem.getShader()));
    }

    @Unique
    private void callWrapperRender(ShaderInstance mcShader) {
        RawShaderProgram rawProgram = GlobalRenderSystem.SHADER_PROGRAM_MAP.get(mcShader.getId());
        ShaderProgram rosellaShaderProgram = Blaze4D.rosella.common.shaderManager.getOrCreateShader(rawProgram);
        wrapper.render(rosellaShaderProgram, GlobalRenderSystem.getShaderUbo(mcShader));
    }

    @Inject(method = "close", at = @At("HEAD"), cancellable = true)
    private void close(CallbackInfo ci) {
        wrapper.clean();
        ci.cancel();
    }
}
