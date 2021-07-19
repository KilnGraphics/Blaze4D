package me.hydos.blaze4d.mixin.shader;

import com.mojang.blaze3d.preprocessor.GlslPreprocessor;
import com.mojang.blaze3d.shaders.Program;
import com.mojang.blaze3d.vertex.VertexFormat;
import it.unimi.dsi.fastutil.objects.ObjectIntPair;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import me.hydos.blaze4d.api.shader.VanillaShaderProcessor;
import net.minecraft.client.renderer.ShaderInstance;
import net.minecraft.server.packs.resources.ResourceProvider;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.ModifyArg;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.util.List;

@Mixin(ShaderInstance.class)
public class ShaderMixin {

    @Inject(method = "<init>", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/renderer/ShaderInstance;parseBlendNode(Lcom/google/gson/JsonObject;)Lcom/mojang/blaze3d/shaders/BlendMode;"))
    public void captureShaderForStaticMethods(ResourceProvider factory, String name, VertexFormat format, CallbackInfo ci) {
        GlobalRenderSystem.blaze4d$capturedShaderProgram = (ShaderAccessor) this;
    }

    @ModifyArg(method = "getOrCreate", at = @At(value = "INVOKE", target = "Lcom/mojang/blaze3d/shaders/Program;compileShader(Lcom/mojang/blaze3d/shaders/Program$Type;Ljava/lang/String;Ljava/io/InputStream;Ljava/lang/String;Lcom/mojang/blaze3d/preprocessor/GlslPreprocessor;)Lcom/mojang/blaze3d/shaders/Program;"), index = 2)
    private static InputStream no(Program.Type type, String name, InputStream stream, String domain, GlslPreprocessor loader) throws IOException {
        String originalSource = new String(stream.readAllBytes());
        ObjectIntPair<List<String>> conversionData = VanillaShaderProcessor.process(
                List.of(originalSource),
                GlobalRenderSystem.blaze4d$capturedShaderProgram.blaze4d$getUniforms(),
                GlobalRenderSystem.processedSamplers,
                GlobalRenderSystem.currentSamplerBinding
        );
        GlobalRenderSystem.currentSamplerBinding = conversionData.valueInt();
        String transformedToVulkan = String.join("\n", conversionData.key());
        return new ByteArrayInputStream(transformedToVulkan.getBytes());
    }
}
