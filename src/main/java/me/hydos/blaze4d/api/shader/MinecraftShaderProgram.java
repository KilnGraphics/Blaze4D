package me.hydos.blaze4d.api.shader;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.stream.Collectors;

import com.google.common.collect.ImmutableMap;
import me.hydos.rosella.device.VulkanDevice;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.resource.Resource;
import me.hydos.rosella.render.shader.RawShaderProgram;
import me.hydos.rosella.memory.Memory;
import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;

import net.minecraft.client.gl.GlUniform;

public class MinecraftShaderProgram extends RawShaderProgram {
    public static final Map<String, MinecraftUbo.AddUboMemoryStep> UBO_MEMORY_STEP_MAP;
    public static Map<Integer, Integer> UNIFORM_SIZES;

    private final List<GlUniform> uniforms;
    private final List<String> samplerNames;

    public MinecraftShaderProgram(@Nullable Resource vertexShader, @Nullable Resource fragmentShader, @NotNull VulkanDevice device, @NotNull Memory memory, int maxObjCount, List<GlUniform> uniforms, List<String> samplers) {
        super(vertexShader, fragmentShader, device, memory, maxObjCount, createPoolTypes(samplers));
        this.uniforms = uniforms;
        this.samplerNames = samplers;
    }

    private static PoolObjType[] createPoolTypes(List<String> samplers) {
        List<PoolObjType> types = new ArrayList<>();
        types.add(PoolObjType.UBO);
        for (int i = 0; i < samplers.size(); i++) {
            types.add(PoolObjType.SAMPLER);
        }
        return types.toArray(PoolObjType[]::new);
    }

    public MinecraftUbo createMinecraftUbo(@NotNull Memory memory, Material material) {
        int size = uniforms.stream().map(GlUniform::getDataType).map(UNIFORM_SIZES::get).reduce(0, Integer::sum);
        List<MinecraftUbo.AddUboMemoryStep> steps = uniforms.stream().map(GlUniform::getName).map(UBO_MEMORY_STEP_MAP::get).collect(Collectors.toList());
        if (steps.contains(null)) {
            throw new RuntimeException("something bad happened: uniforms are " + uniforms);
        }

        return new MinecraftUbo(memory, material, steps, size);
    }

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
                .build();
    }
}
