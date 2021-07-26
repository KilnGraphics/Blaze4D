package me.hydos.blaze4d.api.shader;

import com.mojang.blaze3d.shaders.Uniform;
import it.unimi.dsi.fastutil.ints.Int2IntMap;
import it.unimi.dsi.fastutil.ints.Int2IntMaps;
import it.unimi.dsi.fastutil.ints.Int2IntOpenHashMap;
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
    public static final Int2IntMap UNIFORM_SIZES;

    static {
        Int2IntMap map = new Int2IntOpenHashMap();
        map.put(0, Integer.BYTES);
        map.put(1, 2 * Integer.BYTES);
        map.put(2, 4 * Integer.BYTES);
        map.put(3, 4 * Integer.BYTES);
        map.put(4, Float.BYTES);
        map.put(5, 2 * Float.BYTES);
        map.put(6, 4 * Float.BYTES);
        map.put(7, 4 * Float.BYTES);
        map.put(10, 4 * 4 * Float.BYTES);
        UNIFORM_SIZES = Int2IntMaps.unmodifiable(map);
    }

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
            if (name.equals("DiffuseSampler")) {
                types.add(new PoolSamplerInfo(bindingLocation, -1)); // TODO: set to framebuffer
            } else {
                types.add(new PoolSamplerInfo(bindingLocation, Integer.parseInt(name.substring(7))));
            }
        }

        return types.toArray(PoolObjectInfo[]::new);
    }

    public MinecraftUbo createMinecraftUbo(@NotNull Memory memory, long rawDescriptorPool, ByteBuffer shaderUbo) {
        return new MinecraftUbo(memory, rawDescriptorPool, shaderUbo);
    }
}
