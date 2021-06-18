package me.hydos.blaze4d.api.material;

import me.hydos.blaze4d.api.util.EmptyResource;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.texture.UploadableImage;

public class Blaze4dMaterial extends Material {

    private final UploadableImage image;

    public Blaze4dMaterial(Material old, UploadableImage image) {
        super(EmptyResource.EMPTY, old.getShaderId(), old.getImgFormat(), old.getUseBlend(), old.getTopology(), old.getVertexFormat());
        this.image = image;
    }

    public void loadTextures(Rosella rosella) {
        texture = rosella.getTextureManager().getOrLoadTexture(image, rosella, getImgFormat());
    }
}
