package me.hydos.blaze4d.api;

import it.unimi.dsi.fastutil.objects.Object2ObjectLinkedOpenHashMap;
import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.material.Blaze4dMaterial;
import me.hydos.rosella.render.Topology;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.material.state.StateInfo;
import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.texture.Texture;
import me.hydos.rosella.render.vertex.VertexFormat;

import java.util.Map;

/**
 * Holds all {@link me.hydos.rosella.Rosella} {@link Material}'s used in Blaze4D and Minecraft
 */
public class Materials {

    public static final MaterialBuilder TRIANGLES = register(
            "triangles",
            Topology.TRIANGLES
    );
    public static final MaterialBuilder TRIANGLE_STRIP = register(
            "triangle_strip",
            Topology.TRIANGLE_STRIP
    );
    public static final MaterialBuilder TRIANGLE_FAN = register(
            "triangle_fan",
            Topology.TRIANGLE_FAN
    );
    public static final MaterialBuilder LINES = register(
            "lines",
            Topology.LINE_LIST
    );

    private static final Map<MaterialInfo, Material> MATERIAL_CACHE = new Object2ObjectLinkedOpenHashMap<>();

    public static MaterialBuilder register(String path, Topology topology) {
        return new MaterialBuilder(path, topology);
    }

    public static record MaterialBuilder(String originalPath, Topology topology) {
        public Material build(ShaderProgram shader, Texture[] textures, VertexFormat format, StateInfo stateInfo) {
            return MATERIAL_CACHE.computeIfAbsent(new MaterialInfo(this, shader, textures, format, stateInfo), info -> {
                Blaze4dMaterial material = new Blaze4dMaterial(
                        shader,
                        topology,
                        textures,
                        format,
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
    }

    private record MaterialInfo(
            MaterialBuilder builder,
            ShaderProgram shaderProgram,
            Texture[] textures,
            VertexFormat format,
            StateInfo stateInfo) {
    }
}
