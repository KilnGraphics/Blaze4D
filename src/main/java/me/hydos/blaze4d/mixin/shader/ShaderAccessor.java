package me.hydos.blaze4d.mixin.shader;

import java.util.List;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

import net.minecraft.client.gl.GlUniform;
import net.minecraft.client.render.Shader;

@Mixin(Shader.class)
public interface ShaderAccessor {
    @Accessor(value = "uniforms")
    List<GlUniform> blaze4d$getUniforms();

    @Accessor(value = "samplerNames")
    List<String> blaze4d$getSamplerNames();
}
