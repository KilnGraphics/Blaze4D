package me.hydos.blaze4d.api;

import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.shader.Shaders;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.render.Topology;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.resource.Global;
import me.hydos.rosella.render.resource.Identifier;
import me.hydos.rosella.render.vertex.VertexFormat;
import me.hydos.rosella.render.vertex.VertexFormats;
import org.lwjgl.vulkan.VK10;

import java.awt.image.BufferedImage;

/**
 * Holds all {@link me.hydos.rosella.Rosella} {@link Material}'s used in Blaze4D and Minecraft
 */
public class Materials {

    public static final Material SOLID_TRIANGLES = register(
            "solid_tri",
            Shaders.POSITION,
            Topology.TRIANGLES,
            VertexFormats.Companion.getPOSITION()
    );

    public static final Material SOLID_COLOR_TRIANGLES = register(
            "solid_color_tri",
            Shaders.POSITION_COLOR,
            Topology.TRIANGLES,
            VertexFormats.Companion.getPOSITION()
    );

    public static final Material SOLID_TRIANGLE_STRIP = register(
            "solid_tri_strip",
            Shaders.POSITION,
            Topology.TRIANGLES,
            VertexFormats.Companion.getPOSITION()
    );

    public static final Material SOLID_COLOR_TRIANGLE_STRIP = register(
            "solid_color_tri_strip",
            Shaders.POSITION_COLOR,
            Topology.TRIANGLE_STRIP,
            VertexFormats.Companion.getPOSITION_COLOR()
    );

    public static final Material SOLID_TRIANGLE_FAN = register(
            "solid_tri_fan",
            Shaders.POSITION,
            Topology.TRIANGLES,
            VertexFormats.Companion.getPOSITION()
    );

    public static final Material SOLID_COLOR_TRIANGLE_FAN = register(
            "solid_color_tri_fan",
            Shaders.POSITION_COLOR,
            Topology.TRIANGLE_FAN,
            VertexFormats.Companion.getPOSITION_COLOR()
    );

    public static Material register(String path, Identifier shaderId, Topology topology, VertexFormat format) {
        Identifier id = new Identifier("minecraft", path);
        Material material = new Material(
                Global.INSTANCE.fromBufferedImage(new BufferedImage(1, 1, BufferedImage.TYPE_4BYTE_ABGR), id),
                shaderId,
                VK10.VK_FORMAT_R8G8B8A8_UNORM,
                false,
                topology,
                format
        );
        Blaze4D.rosella.registerMaterial(id, material);
        return material;
    }

    public static void initialize(Rosella rosella) {
        rosella.reloadMaterials();
    }
}
