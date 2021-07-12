package me.hydos.blaze4d.api.shader;

import com.google.common.collect.ImmutableMap;
import me.hydos.rosella.device.VulkanDevice;
import me.hydos.rosella.memory.Memory;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.resource.Resource;
import me.hydos.rosella.render.shader.RawShaderProgram;
import net.minecraft.client.gl.GlUniform;
import net.minecraft.util.math.MathHelper;
import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.stream.Collectors;

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

    private final List<GlUniform> uniforms;
    private final List<String> samplerNames;

    public MinecraftShaderProgram(@Nullable Resource vertexShader, @Nullable Resource fragmentShader, @NotNull VulkanDevice device, @NotNull Memory memory, int maxObjCount, List<GlUniform> uniforms, List<String> samplerNames) {
        super(vertexShader, fragmentShader, device, memory, maxObjCount, createPoolTypes(samplerNames));
        this.uniforms = uniforms;
        this.samplerNames = samplerNames;
    }

    private static PoolObjectInfo[] createPoolTypes(List<String> samplerNames) {
        List<PoolObjectInfo> types = new ArrayList<>();
        types.add(PoolUboInfo.INSTANCE);

        for (String name : samplerNames) {
            if (name.equals("DiffuseSampler")) {
                types.add(new PoolSamplerInfo(-1)); // TODO: set to framebuffer
            } else {
                types.add(new PoolSamplerInfo(Integer.parseInt(name.substring(7))));
            }
        }

        return types.toArray(PoolObjectInfo[]::new);
    }

    public MinecraftUbo createMinecraftUbo(@NotNull Memory memory, Material material) {
        List<MinecraftUbo.AddUboMemoryStep> steps = new ArrayList<>();
        int size = 0;

        for (GlUniform uniform : uniforms) {
            MinecraftUbo.AddUboMemoryStep step = UBO_MEMORY_STEP_MAP.get(uniform.getName());

            if (step == null) {
                throw new RuntimeException("something bad happened: uniforms are " + uniforms);
            }

            size += UNIFORM_SIZES.get(uniform.getDataType());
            steps.add(step);
        }

        return new MinecraftUbo(memory, material, steps, MathHelper.roundUpToMultiple(size, 16));
    }
}
