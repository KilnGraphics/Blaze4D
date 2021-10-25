package graphics.kiln.blaze4d.mixin.shader;

import com.mojang.blaze3d.systems.RenderSystem;
import graphics.kiln.blaze4d.Blaze4D;
import graphics.kiln.blaze4d.impl.GlobalRenderSystem;
import graphics.kiln.rosella.render.shader.RawShaderProgram;
import net.minecraft.client.renderer.ShaderInstance;
import org.jetbrains.annotations.Nullable;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Shadow;

import java.util.function.Supplier;

@Mixin(value = RenderSystem.class, remap = false)
public abstract class RenderSystemMixin {

    @Shadow
    @Nullable
    private static ShaderInstance shader;

    /**
     * @author Blaze4D
     * @reason To Integrate Shader Programs
     */
    @Overwrite
    public static void setShader(Supplier<ShaderInstance> supplier) {
        ShaderInstance result = supplier.get();
        if (result == null) {
            return;
        }
        RenderSystemMixin.shader = result;
        RawShaderProgram rawProgram = GlobalRenderSystem.SHADER_PROGRAM_MAP.get(RenderSystemMixin.shader.getId());
        GlobalRenderSystem.activeShader = Blaze4D.rosella.common.shaderManager.getOrCreateShader(rawProgram);
    }
}
