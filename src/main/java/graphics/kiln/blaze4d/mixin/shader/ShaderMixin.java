package graphics.kiln.blaze4d.mixin.shader;

import com.google.common.collect.Lists;
import com.mojang.blaze3d.preprocessor.GlslPreprocessor;
import com.mojang.blaze3d.shaders.Program;
import com.mojang.blaze3d.shaders.Uniform;
import com.mojang.blaze3d.systems.RenderSystem;
import com.mojang.blaze3d.vertex.VertexFormat;
import com.mojang.math.Matrix4f;
import com.mojang.math.Vector3f;
import graphics.kiln.blaze4d.Blaze4D;
import graphics.kiln.blaze4d.api.B4DShader;
import graphics.kiln.blaze4d.api.B4DUniform;
import graphics.kiln.blaze4d.api.Utils;
import graphics.kiln.blaze4d.core.McUniform;
import graphics.kiln.blaze4d.core.natives.VertexFormatNative;
import net.minecraft.client.renderer.ShaderInstance;
import net.minecraft.server.packs.resources.ResourceProvider;
import org.spongepowered.asm.mixin.Final;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.ModifyArg;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

@Mixin(ShaderInstance.class)
public class ShaderMixin implements B4DShader {

    @Final
    @Shadow
    private List<Uniform> uniforms;

    @Final
    private long b4dShaderId = 0L;

    @Inject(method = "<init>", at = @At(value = "TAIL"))
    private void initShader(ResourceProvider resourceProvider, String string, VertexFormat vertexFormat, CallbackInfo ci) {
        try (VertexFormatNative nativeFormat = new VertexFormatNative()) {
            long usedUniforms = 0L;
            for (Uniform uniform : this.uniforms) {
                McUniform mcUniform = ((B4DUniform) uniform).getMcUniform();
                if (mcUniform != null) {
                    usedUniforms |= mcUniform.getValue();
                }
            }

            if (Utils.convertVertexFormat(vertexFormat, nativeFormat)) {
                this.b4dShaderId = Blaze4D.core.createShader(nativeFormat, usedUniforms);
            } else {
                Blaze4D.LOGGER.warn("Shader vertex format did not contain position. Skipping!");
            }
        } catch (Exception e) {
            throw new RuntimeException("Failed to create shader", e);
        }
    }

    @Inject(method = "apply", at = @At(value = "TAIL"))
    private void applyShader(CallbackInfo ci) {
        Matrix4f proj = RenderSystem.getProjectionMatrix();
        Matrix4f modelView = RenderSystem.getModelViewMatrix();
    }

    @Inject(method = "close", at = @At(value = "TAIL"))
    private void destroyShader(CallbackInfo ci) {
        Blaze4D.core.destroyShader(this.b4dShaderId);
    }

    @Override
    public long b4dGetShaderId() {
        return this.b4dShaderId;
    }

//
//    @ModifyArg(method = "getOrCreate", at = @At(value = "INVOKE", target = "Lcom/mojang/blaze3d/shaders/Program;compileShader(Lcom/mojang/blaze3d/shaders/Program$Type;Ljava/lang/String;Ljava/io/InputStream;Ljava/lang/String;Lcom/mojang/blaze3d/preprocessor/GlslPreprocessor;)Lcom/mojang/blaze3d/shaders/Program;"), index = 2)
//    private static InputStream no(Program.Type type, String name, InputStream stream, String domain, GlslPreprocessor loader) throws IOException {
//        String originalSource = new String(stream.readAllBytes());
//        Rosella.LOGGER.debug("Processing shader " + name + type.getExtension());
//        Map<String, Integer> uniforms = new LinkedHashMap<>();
//
//        for (Uniform uniform : GlobalRenderSystem.blaze4d$capturedShaderProgram.blaze4d$getUniforms()) {
//            if (uniforms.put(uniform.getName(), uniform.getType()) != null) {
//                throw new IllegalStateException("Duplicate key");
//            }
//        }
//
//        VanillaShaderProcessor.ConversionData conversionData = VanillaShaderProcessor.process(
//                List.of(originalSource),
//                uniforms,
//                GlobalRenderSystem.processedSamplers,
//                GlobalRenderSystem.currentSamplerBinding
//        );
//
//        GlobalRenderSystem.currentSamplerBinding = conversionData.samplerBinding();
//        String transformedToVulkan = String.join("\n", conversionData.lines());
//        return new ByteArrayInputStream(transformedToVulkan.getBytes());
//    }
//
//    @Inject(method = "<init>", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/renderer/ShaderInstance;parseBlendNode(Lcom/google/gson/JsonObject;)Lcom/mojang/blaze3d/shaders/BlendMode;"))
//    public void captureShaderForStaticMethods(ResourceProvider factory, String name, VertexFormat format, CallbackInfo ci) {
//        GlobalRenderSystem.blaze4d$capturedShaderProgram = (ShaderAccessor) this;
//    }
}
