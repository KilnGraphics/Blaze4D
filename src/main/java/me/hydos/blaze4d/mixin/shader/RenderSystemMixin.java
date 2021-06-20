package me.hydos.blaze4d.mixin.shader;

import com.mojang.blaze3d.systems.RenderSystem;
import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.VkRenderSystem;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.render.shader.RawShaderProgram;
import net.minecraft.client.render.Shader;
import org.jetbrains.annotations.Nullable;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;

import java.util.function.Supplier;

@Mixin(RenderSystem.class)
public abstract class RenderSystemMixin {

    @Shadow
    @Nullable
    private static Shader shader;

    /**
     * @author Blaze4D
     * @reason To Integrate Shader Programs
     */
    @Overwrite
    public static void setShader(Supplier<Shader> supplier) {
        Shader result = supplier.get();
        if(result == null) {
            return;
        }
        if (!RenderSystem.isOnRenderThread()) {
            RenderSystem.recordRenderCall(() -> {
                RenderSystemMixin.shader = result;
                RawShaderProgram rawProgram = VkRenderSystem.SHADER_PROGRAM_MAP.get(RenderSystemMixin.shader.getProgramRef());
                VkRenderSystem.activeShader = Blaze4D.rosella.getShaderManager().getOrCreateShader(rawProgram);
            });
        } else {
            RenderSystemMixin.shader = result;
            RawShaderProgram rawProgram = VkRenderSystem.SHADER_PROGRAM_MAP.get(RenderSystemMixin.shader.getProgramRef());
            VkRenderSystem.activeShader = Blaze4D.rosella.getShaderManager().getOrCreateShader(rawProgram);
        }

    }
}
