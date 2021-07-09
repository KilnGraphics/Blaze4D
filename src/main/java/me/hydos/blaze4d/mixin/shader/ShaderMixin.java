package me.hydos.blaze4d.mixin.shader;

import me.hydos.blaze4d.api.GlobalRenderSystem;
import me.hydos.blaze4d.api.shader.OpenGLToVulkanShaderProcessor;
import net.minecraft.client.render.Shader;
import net.minecraft.client.render.VertexFormat;
import net.minecraft.resource.ResourceFactory;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.ModifyArg;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.util.List;

@Mixin(Shader.class)
public class ShaderMixin {

    @Inject(method = "<init>", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/render/Shader;readBlendState(Lcom/google/gson/JsonObject;)Lnet/minecraft/client/gl/GlBlendState;"))
    public void captureShaderForStaticMethods(ResourceFactory factory, String name, VertexFormat format, CallbackInfo ci) {
        GlobalRenderSystem.blaze4d$capturedShader = (ShaderAccessor) this;
    }

    @ModifyArg(method = "loadProgram", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/gl/Program;createFromResource(Lnet/minecraft/client/gl/Program$Type;Ljava/lang/String;Ljava/io/InputStream;Ljava/lang/String;Lnet/minecraft/client/gl/GLImportProcessor;)Lnet/minecraft/client/gl/Program;"), index = 2)
    private static InputStream no(InputStream stream) throws IOException {
        String originalSource = new String(stream.readAllBytes());
        String transformedToVulkan = String.join("\n", OpenGLToVulkanShaderProcessor.convertOpenGLToVulkanShader(List.of(originalSource), GlobalRenderSystem.blaze4d$capturedShader.blaze4d$getUniforms()));
        return new ByteArrayInputStream(transformedToVulkan.getBytes());
    }
}
