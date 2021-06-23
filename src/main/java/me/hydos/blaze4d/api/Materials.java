package me.hydos.blaze4d.api;

import it.unimi.dsi.fastutil.objects.Object2ObjectLinkedOpenHashMap;
import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.material.Blaze4dMaterial;
import me.hydos.rosella.render.Topology;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.resource.Identifier;
import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.texture.UploadableImage;
import me.hydos.rosella.render.vertex.VertexFormat;
import org.lwjgl.vulkan.VK10;

import java.util.Map;

/**
 * Holds all {@link me.hydos.rosella.Rosella} {@link Material}'s used in Blaze4D and Minecraft
 */
public class Materials {

    private static final Map<MaterialInfo, Material> MATERIAL_CACHE = new Object2ObjectLinkedOpenHashMap<>();

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

    public static MaterialBuilder register(String path, Topology topology) {
        return new MaterialBuilder(path, topology);
    }

    public static record MaterialBuilder(String originalPath, Topology topology) {
        public Material build(ShaderProgram shader, UploadableImage image, VertexFormat format) {
            return MATERIAL_CACHE.computeIfAbsent(new MaterialInfo(this, shader, image, format), info -> {
                Blaze4dMaterial material = new Blaze4dMaterial(
                        shader,
                        VK10.VK_FORMAT_R8G8B8A8_UNORM,
                        false,
                        topology,
                        format,
                        image
                );
                Blaze4D.rosella.registerMaterial(
                        new Identifier("minecraft", originalPath + "_" + shader.hashCode() + "_" + format.hashCode()),
                        material
                );
                Blaze4D.rosella.getRenderer().clearCommandBuffers();
                Blaze4D.rosella.reloadMaterials();
                return material;
            });
        }
    }

    private record MaterialInfo(MaterialBuilder builder, ShaderProgram shaderProgram, UploadableImage image,
                                VertexFormat format) {
    }
}
