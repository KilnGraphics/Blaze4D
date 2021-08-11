package me.hydos.blaze4d.api.shader;

import com.mojang.blaze3d.shaders.Uniform;
import it.unimi.dsi.fastutil.objects.Object2IntMap;
import me.hydos.rosella.device.VulkanDevice;
import me.hydos.rosella.memory.Memory;
import me.hydos.rosella.render.resource.Resource;
import me.hydos.rosella.render.shader.RawShaderProgram;
import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;

import java.nio.ByteBuffer;
import java.util.ArrayList;
import java.util.List;

public class MinecraftShaderProgram extends RawShaderProgram {
    
    private final List<Uniform> uniforms;
    private final Object2IntMap<String> samplers;

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
