package me.hydos.rosella.render.material;

import me.hydos.rosella.Rosella;
import me.hydos.rosella.render.Topology;
import me.hydos.rosella.render.material.state.StateInfo;
import me.hydos.rosella.render.resource.Resource;
import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.texture.ImageFormat;
import me.hydos.rosella.render.texture.SamplerCreateInfo;
import me.hydos.rosella.render.texture.StbiImage;
import me.hydos.rosella.render.texture.Texture;
import me.hydos.rosella.render.texture.TextureManager;
import me.hydos.rosella.render.texture.UploadableImage;
import me.hydos.rosella.render.vertex.VertexFormat;
import me.hydos.rosella.scene.object.impl.SimpleObjectManager;

/**
 * A Material is like texture information, normal information, and all of those things which give an object character wrapped into one class.
 * similar to how unity material's works
 * guaranteed to change in the future
 */
public class Material {

    protected final Resource resource;
    protected final ShaderProgram shader;
    protected final int imgFormat;
    protected final Topology topology;
    protected final VertexFormat vertexFormat;
    protected final SamplerCreateInfo samplerCreateInfo;
    protected final StateInfo stateInfo;

    public Material(Resource resource, ShaderProgram shader, int imgFormat, Topology topology, VertexFormat vertexFormat, SamplerCreateInfo samplerCreateInfo, StateInfo stateInfo) {
        this.resource = resource;
        this.shader = shader;
        this.imgFormat = imgFormat;
        this.topology = topology;
        this.vertexFormat = vertexFormat;
        this.samplerCreateInfo = samplerCreateInfo;
        this.stateInfo = stateInfo;
    }

    protected PipelineInfo pipeline;
    protected Texture[] textures;

    public void loadTextures(SimpleObjectManager objectManager, Rosella rosella) { //FIXME this is also temporary
        if (resource != Resource.Empty.INSTANCE) {
            TextureManager textureManager = objectManager.textureManager;
            int textureId = textureManager.generateTextureId(); // FIXME this texture can't be removed
            UploadableImage image = new StbiImage(resource, ImageFormat.fromVkFormat(imgFormat));
            textureManager.createTexture(
                    rosella.renderer,
                    textureId,
                    image.getWidth(),
                    image.getHeight(),
                    imgFormat
            );
            textureManager.setTextureSampler(
                    textureId,
                    0,
                    samplerCreateInfo
            ); // 0 is the default texture no, but it's still gross
            textureManager.drawToExistingTexture(rosella.renderer, rosella.common.memory, textureId, image);
            Texture texture = textureManager.getTexture(textureId);
            textures = new Texture[]{texture}; //FIXME THIS SUCKS
        }
    }

    public ShaderProgram getShader() {
        return shader;
    }

    public PipelineInfo getPipeline() {
        return pipeline;
    }

    public void setPipeline(PipelineInfo pipeline) {
        this.pipeline = pipeline;
    }

    public Texture[] getTextures() {
        return textures;
    }
}

