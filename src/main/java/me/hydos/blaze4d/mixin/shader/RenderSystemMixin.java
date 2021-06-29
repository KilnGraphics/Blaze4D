package me.hydos.blaze4d.mixin.shader;

import com.mojang.blaze3d.systems.RenderSystem;
import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import me.hydos.blaze4d.api.shader.MinecraftUbo;
import me.hydos.rosella.render.shader.RawShaderProgram;
import net.minecraft.client.render.Shader;
import net.minecraft.client.util.math.MatrixStack;
import net.minecraft.util.math.Matrix4f;
import org.jetbrains.annotations.Nullable;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.util.function.Supplier;

@Mixin(value = RenderSystem.class, remap = false)
public abstract class RenderSystemMixin {

    @Shadow
    @Nullable
    private static Shader shader;

    @Shadow
    private static MatrixStack modelViewStack;

    /**
     * @author Blaze4D
     * @reason To Integrate Shader Programs
     */
    @Overwrite
    public static void setShader(Supplier<Shader> supplier) {
        Shader result = supplier.get();
        if (result == null) {
            return;
        }
        if (!RenderSystem.isOnRenderThread()) {
            RenderSystem.recordRenderCall(() -> {
                RenderSystemMixin.shader = result;
                RawShaderProgram rawProgram = GlobalRenderSystem.SHADER_PROGRAM_MAP.get(RenderSystemMixin.shader.getProgramRef());
                GlobalRenderSystem.activeShader = Blaze4D.rosella.getShaderManager().getOrCreateShader(rawProgram);
            });
        } else {
            RenderSystemMixin.shader = result;
            RawShaderProgram rawProgram = GlobalRenderSystem.SHADER_PROGRAM_MAP.get(RenderSystemMixin.shader.getProgramRef());
            GlobalRenderSystem.activeShader = Blaze4D.rosella.getShaderManager().getOrCreateShader(rawProgram);
        }
    }

    @Inject(method = "applyModelViewMatrix", at = @At("HEAD"))
    private static void setOurModelViewMatrix(CallbackInfo ci) {
        GlobalRenderSystem.modelViewMatrix = MinecraftUbo.toJoml(modelViewStack.peek().getModel());
    }
}
