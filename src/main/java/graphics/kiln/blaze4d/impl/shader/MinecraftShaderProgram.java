package graphics.kiln.blaze4d.impl.shader;

import com.mojang.blaze3d.shaders.Uniform;
import it.unimi.dsi.fastutil.objects.Object2IntMap;
import graphics.kiln.rosella.device.VulkanDevice;
import graphics.kiln.rosella.memory.Memory;
import graphics.kiln.rosella.render.resource.Resource;
import graphics.kiln.rosella.render.shader.RawShaderProgram;
import graphics.kiln.blaze4d.impl.ubo.MinecraftUbo;
import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;

import java.nio.ByteBuffer;
import java.util.ArrayList;
import java.util.List;

/**
 * Special version of RawShaderProgram to work better with Minecraft's Shader system
 */
public class MinecraftShaderProgram extends RawShaderProgram {
    
    protected final List<Uniform> uniforms;
    protected final Object2IntMap<String> samplers;

    public MinecraftShaderProgram(@Nullable Resource vertexShader, @Nullable Resource fragmentShader, @NotNull VulkanDevice device, @NotNull Memory memory, int maxObjCount, List<Uniform> uniforms, Object2IntMap<String> samplers) {
        super(vertexShader, fragmentShader, device, memory, maxObjCount, createPoolTypes(samplers));
        this.uniforms = uniforms;
        this.samplers = samplers;
    }

    private static PoolObjectInfo[] createPoolTypes(Object2IntMap<String> samplers) {
        List<PoolObjectInfo> types = new ArrayList<>();
        types.add(PoolUboInfo.INSTANCE);

        for (Object2IntMap.Entry<String> sampler : samplers.object2IntEntrySet()) {
            String name = sampler.getKey();
            int bindingLocation = sampler.getIntValue();
            types.add(new PoolSamplerInfo(bindingLocation, name));
        }

        return types.toArray(PoolObjectInfo[]::new);
    }

    public MinecraftUbo createMinecraftUbo(@NotNull Memory memory, long rawDescriptorPool, ByteBuffer shaderUbo) {
        return new MinecraftUbo(memory, rawDescriptorPool, shaderUbo);
    }
}
