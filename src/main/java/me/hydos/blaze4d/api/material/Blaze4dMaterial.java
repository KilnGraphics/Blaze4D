package me.hydos.blaze4d.api.material;

import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.util.EmptyResource;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.render.Topology;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.resource.Identifier;
import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.texture.SamplerCreateInfo;
import me.hydos.rosella.render.texture.Texture;
import me.hydos.rosella.render.texture.TextureFilter;
import me.hydos.rosella.render.texture.WrapMode;
import me.hydos.rosella.render.vertex.VertexFormat;
import me.hydos.rosella.scene.object.impl.SimpleObjectManager;
import net.minecraft.client.texture.MissingSprite;
import net.minecraft.client.texture.NativeImageBackedTexture;
import org.jetbrains.annotations.NotNull;

public class Blaze4dMaterial extends Material {

    private static final Texture missingTexture = createMissingTexture();

    private static Texture createMissingTexture() {
        NativeImageBackedTexture missingTex = MissingSprite.getMissingSpriteTexture();
        missingTex.getImage().upload(0, 0, 0, false);
        return ((SimpleObjectManager) Blaze4D.rosella.objectManager).textureManager.getTexture(missingTex.getGlId());
    }

    private final int textureId;

    public Blaze4dMaterial(ShaderProgram shader, int imageFormat, boolean useBlend, Topology topology, VertexFormat format, int textureId) {
        super(EmptyResource.EMPTY, null, imageFormat, useBlend, topology, format, new SamplerCreateInfo(TextureFilter.NEAREST, WrapMode.REPEAT));
        this.textureId = textureId;
        this.setShader(shader);
    }

    @Override
    public void loadTextures(SimpleObjectManager objectManager, Rosella rosella) {
        Texture retrievedTexture = ((SimpleObjectManager) Blaze4D.rosella.objectManager).textureManager.getTexture(textureId);
        texture = retrievedTexture == null ? missingTexture : retrievedTexture;
    }
}
