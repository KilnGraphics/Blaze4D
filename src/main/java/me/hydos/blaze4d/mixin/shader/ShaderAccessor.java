package me.hydos.blaze4d.mixin.shader;

import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;
import com.mojang.blaze3d.shaders.Uniform;
import java.util.List;
import net.minecraft.client.renderer.ShaderInstance;

@Mixin(ShaderInstance.class)
public interface ShaderAccessor {

    @Accessor(value = "uniforms")
    List<Uniform> blaze4d$getUniforms();
}
