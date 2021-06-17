package me.hydos.blaze4d.api.material;

import me.hydos.rosella.Rosella;
import me.hydos.rosella.render.Topology;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.resource.Identifier;
import me.hydos.rosella.render.resource.Resource;
import me.hydos.rosella.render.texture.UploadableImage;
import org.jetbrains.annotations.NotNull;
import org.lwjgl.vulkan.VK10;

public class Blaze4dMaterial extends Material {

    private final UploadableImage image;

    public Blaze4dMaterial(@NotNull Resource resource, @NotNull Identifier shaderId, int imgFormat, boolean useBlend, @NotNull Topology topology, UploadableImage image) {
        super(resource, shaderId, imgFormat, useBlend, topology);
        this.image = image;
    }

    @Override
    public void loadTextures(@NotNull Rosella rosella) {
        this.texture = rosella.getTextureManager().getOrLoadTexture(image, rosella, VK10.VK_FORMAT_R8G8B8A8_SINT);
    }
}
