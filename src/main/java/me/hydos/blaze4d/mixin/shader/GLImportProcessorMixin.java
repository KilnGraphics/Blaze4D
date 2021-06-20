package me.hydos.blaze4d.mixin.shader;

import me.hydos.blaze4d.api.shader.OpenGLToVulkanShaderProcessor;
import net.minecraft.client.gl.GLImportProcessor;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import java.util.List;

@Mixin(GLImportProcessor.class)
public class GLImportProcessorMixin {

    /**
     * Sets locations so the SPIR-V compiler doesnt complain
     * @param source the source of the shader
     * @param cir the original return value
     */
    @Inject(method = "readSource", at = @At("RETURN"), cancellable = true)
    private void setLocations(String source, CallbackInfoReturnable<List<String>> cir) {
        cir.setReturnValue(OpenGLToVulkanShaderProcessor.convertOpenGLToVulkanShader(cir.getReturnValue()));
    }
}
