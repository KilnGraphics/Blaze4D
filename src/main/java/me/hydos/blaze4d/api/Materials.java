package me.hydos.blaze4d.api;

import me.hydos.blaze4d.Blaze4D;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.resource.Global;
import me.hydos.rosella.render.resource.Identifier;
import org.lwjgl.vulkan.VK10;

import java.awt.image.BufferedImage;
import java.nio.ByteBuffer;

/**
 * Holds all {@link me.hydos.rosella.Rosella} {@link Material}'s used in Blaze4D and Minecraft
 */
public class Materials {

    public static final Material SOLID_COLOR = register("solid", Shaders.POSITION_COLOR);

    public static Material register(String path, Identifier shaderId) {
        Identifier id = new Identifier("minecraft", path);
        Material material = new Material(
                Global.INSTANCE.fromBufferedImage(new BufferedImage(1, 1, BufferedImage.TYPE_4BYTE_ABGR), id),
                shaderId,
                VK10.VK_FORMAT_R8G8B8A8_UNORM,
                false
        );
        Blaze4D.rosella.registerMaterial(id, material);
        return material;
    }

    public static void initialize(Rosella rosella) {
        rosella.reloadMaterials();
    }
}
