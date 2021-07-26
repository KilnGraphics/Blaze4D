package me.hydos.blaze4d.api;

import it.unimi.dsi.fastutil.objects.Object2ObjectLinkedOpenHashMap;
import me.hydos.blaze4d.Blaze4D;
import me.hydos.rosella.render.Topology;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.material.state.StateInfo;
import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.vertex.VertexFormat;

import java.util.Map;

/**
 * Holds all {@link me.hydos.rosella.Rosella} {@link Material}'s used in Blaze4D and Minecraft
 */
public class Materials {

    private static final Map<MaterialInfo, Material> MATERIAL_CACHE = new Object2ObjectLinkedOpenHashMap<>();

    public static Material getOrCreateMaterial(Topology topology, ShaderProgram shaderProgram, VertexFormat format, StateInfo stateInfo) {
        return MATERIAL_CACHE.computeIfAbsent(new MaterialInfo(topology, shaderProgram, format, stateInfo), info -> {
            Material material = new Material(
                    info.shaderProgram,
                    info.topology,
                    info.format,
                    info.stateInfo
            );
            Blaze4D.rosella.objectManager.registerMaterial(
                    material
            );
            Blaze4D.rosella.renderer.clearCommandBuffers(Blaze4D.rosella.common.device);
            Blaze4D.rosella.objectManager.submitMaterials();
            return material;
        });
    }

    private record MaterialInfo(
            Topology topology,
            ShaderProgram shaderProgram,
            VertexFormat format,
            StateInfo stateInfo) {
    }
}
