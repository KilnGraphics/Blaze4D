package me.hydos.blaze4d.api.shader;

import com.google.common.collect.ImmutableMap;
import com.mojang.blaze3d.shaders.Uniform;
import it.unimi.dsi.fastutil.objects.Object2IntMap;
import it.unimi.dsi.fastutil.objects.Object2IntOpenHashMap;
import me.hydos.rosella.device.VulkanDevice;
import me.hydos.rosella.memory.Memory;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.resource.Resource;
import me.hydos.rosella.render.shader.RawShaderProgram;
import me.hydos.rosella.render.shader.ShaderType;
import net.minecraft.util.Mth;
import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;

public class MinecraftShaderProgram extends RawShaderProgram {

    public static final Map<String, MinecraftUbo.AddUboMemoryStep> UBO_MEMORY_STEP_MAP;
    public static final Map<Integer, Integer> UNIFORM_SIZES;

    static {
        UBO_MEMORY_STEP_MAP = new ImmutableMap.Builder<String, MinecraftUbo.AddUboMemoryStep>()
                .put("ModelViewMat", MinecraftUbo::addViewTransformMatrix)
                .put("ProjMat", MinecraftUbo::addProjectionMatrix)
                .put("ColorModulator", MinecraftUbo::addShaderColor)
                .put("FogStart", MinecraftUbo::addFogStart)
                .put("FogEnd", MinecraftUbo::addFogEnd)
                .put("FogColor", MinecraftUbo::addFogColor)
                .put("TextureMat", MinecraftUbo::addTextureMatrix)
                .put("GameTime", MinecraftUbo::addGameTime)
                .put("ScreenSize", MinecraftUbo::addScreenSize)
                .put("LineWidth", MinecraftUbo::addLineWidth)
                .put("ChunkOffset", MinecraftUbo::addChunkOffset)
                .put("Light0_Direction", MinecraftUbo::addLightDirections0)
                .put("Light1_Direction", MinecraftUbo::addLightDirections1)
                .put("EndPortalLayers", MinecraftUbo::addEndPortalLayers)
                .build();

        UNIFORM_SIZES = new ImmutableMap.Builder<Integer, Integer>()
                .put(0, Integer.BYTES)
                .put(1, 2 * Integer.BYTES)
                .put(2, 3 * Integer.BYTES)
                .put(3, 4 * Integer.BYTES)
                .put(4, Float.BYTES)
                .put(5, 2 * Float.BYTES)
                .put(6, 3 * Float.BYTES)
                .put(7, 4 * Float.BYTES)
                .put(10, 4 * 4 * Float.BYTES)
                .put(11, Integer.BYTES)
                .build();
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

    public MinecraftUbo createMinecraftUbo(@NotNull Memory memory, Material material) {
        List<MinecraftUbo.AddUboMemoryStep> steps = new ArrayList<>();
        int size = 0;

        for (Uniform uniform : uniforms) {
            MinecraftUbo.AddUboMemoryStep step = UBO_MEMORY_STEP_MAP.get(uniform.getName());

            if (step == null) {
                throw new RuntimeException("something bad happened: uniforms are " + uniforms);
            }

            size += UNIFORM_SIZES.get(uniform.getType());
            steps.add(step);
        }

        return new MinecraftUbo(memory, material, steps, Mth.roundToward(size, 16));
    }
}
