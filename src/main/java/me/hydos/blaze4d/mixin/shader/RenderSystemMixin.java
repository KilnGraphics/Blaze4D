package me.hydos.blaze4d.mixin.shader;

import com.mojang.blaze3d.systems.RenderSystem;
import com.mojang.blaze3d.vertex.PoseStack;
import com.mojang.math.Vector3f;
import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import me.hydos.blaze4d.api.shader.MinecraftUbo;
import me.hydos.blaze4d.api.util.ConversionUtils;
import me.hydos.rosella.render.shader.RawShaderProgram;
import me.hydos.rosella.scene.object.impl.SimpleObjectManager;
import net.minecraft.client.renderer.ShaderInstance;
import org.jetbrains.annotations.Nullable;
import org.spongepowered.asm.mixin.Final;
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
    private static ShaderInstance shader;

    @Shadow
    private static PoseStack modelViewStack;

    @Shadow
    @Final
    private static Vector3f[] shaderLightDirections;

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
        GlobalRenderSystem.activeShader = ((SimpleObjectManager) Blaze4D.rosella.objectManager).shaderManager.getOrCreateShader(rawProgram);
    }

    @Inject(method = "applyModelViewMatrix", at = @At("HEAD"))
    private static void setOurModelViewMatrix(CallbackInfo ci) {
        GlobalRenderSystem.modelViewMatrix = ConversionUtils.mcToJomlMatrix(modelViewStack.last().pose());
    }

    @Inject(method = "setupShaderLights", at = @At("TAIL"))
    private static void setShaderLights(ShaderInstance shader, CallbackInfo ci) {
        GlobalRenderSystem.shaderLightDirections0 = shaderLightDirections[0];
        GlobalRenderSystem.shaderLightDirections1 = shaderLightDirections[1];
    }
}
