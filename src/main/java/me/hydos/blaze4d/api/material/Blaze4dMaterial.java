package me.hydos.blaze4d.api.material;

import me.hydos.blaze4d.api.util.EmptyResource;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.render.Topology;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.material.state.StateInfo;
import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.texture.SamplerCreateInfo;
import me.hydos.rosella.render.texture.Texture;
import me.hydos.rosella.render.texture.TextureFilter;
import me.hydos.rosella.render.texture.WrapMode;
import me.hydos.rosella.render.vertex.VertexFormat;
import me.hydos.rosella.scene.object.impl.SimpleObjectManager;
import org.lwjgl.vulkan.VK10;

public class Blaze4dMaterial extends Material {

    public Blaze4dMaterial(ShaderProgram shader, Topology topology, Texture[] textures, VertexFormat format, StateInfo stateInfo) {
        // fill the elements of the parent that we don't need with random stuff
        super(EmptyResource.EMPTY, shader, VK10.VK_FORMAT_R32G32B32A32_SFLOAT, topology, format, new SamplerCreateInfo(TextureFilter.NEAREST, WrapMode.REPEAT), stateInfo);
        this.textures = textures;
    }

    @Override
    public void loadTextures(SimpleObjectManager objectManager, Rosella rosella) {
        // noop
    }
}
