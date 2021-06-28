package me.hydos.blaze4d.api.material;

import me.hydos.blaze4d.api.util.EmptyResource;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.render.Topology;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.resource.Identifier;
import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.texture.UploadableImage;
import me.hydos.rosella.render.vertex.VertexFormat;
import org.jetbrains.annotations.NotNull;
import org.lwjgl.vulkan.VK10;

public class Blaze4dMaterial extends Material {

    private final UploadableImage image;

    public Blaze4dMaterial(Material old, UploadableImage image) {
        super(EmptyResource.EMPTY, old.getShaderId(), old.getImgFormat(), old.getUseBlend(), old.getTopology(), old.getVertexFormat(), VK10.VK_FILTER_NEAREST);
        this.image = image;
        this.shader = old.shader;
        if (this.shader == null) {
            throw new RuntimeException("Shader is Null");
        }
    }

    public Blaze4dMaterial(ShaderProgram shader, int imageFormat, boolean useBlend, Topology topology, VertexFormat format, UploadableImage image) {
        super(EmptyResource.EMPTY, Identifier.getEMPTY(), imageFormat, useBlend, topology, format, VK10.VK_FILTER_NEAREST);
        this.image = image;
        this.shader = shader;
    }

    @Override
    public void loadShaders(@NotNull Rosella engine) {
    }

    public void loadTextures(Rosella rosella) {
        texture = rosella.getTextureManager().getOrLoadTexture(image, rosella, getImgFormat(), VK10.VK_FILTER_NEAREST);
    }
}
