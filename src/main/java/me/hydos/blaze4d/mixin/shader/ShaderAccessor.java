package me.hydos.blaze4d.mixin.shader;

import net.minecraft.client.gl.GlUniform;
import net.minecraft.client.render.Shader;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

import java.util.List;

@Mixin(Shader.class)
public interface ShaderAccessor {

    @Accessor(value = "uniforms")
    List<GlUniform> blaze4d$getUniforms();
}
