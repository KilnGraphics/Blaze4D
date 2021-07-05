package me.hydos.rosella.scene.object.impl;

import it.unimi.dsi.fastutil.objects.Object2ObjectArrayMap;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.display.Display;
import me.hydos.rosella.render.info.InstanceInfo;
import me.hydos.rosella.render.info.RenderInfo;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.material.PipelineManager;
import me.hydos.rosella.render.renderer.Renderer;
import me.hydos.rosella.render.shader.RawShaderProgram;
import me.hydos.rosella.render.shader.ShaderManager;
import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.swapchain.RenderPass;
import me.hydos.rosella.render.texture.TextureManager;
import me.hydos.rosella.scene.object.ObjectManager;
import me.hydos.rosella.scene.object.Renderable;
import me.hydos.rosella.vkobjects.VkCommon;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;

/**
 * Just a basic object manager
 */
public class SimpleObjectManager implements ObjectManager {

    private final Renderer renderer;
    public final ShaderManager shaderManager;
    public final TextureManager textureManager;
    public final PipelineManager pipelineManager;
    public final Map<RenderInfo, List<InstanceInfo>> renderObjects = new Object2ObjectArrayMap<>();

    public final List<Material> materials = new ArrayList<>();
    public final List<Material> unprocessedMaterials = new ArrayList<>();

    public SimpleObjectManager(Rosella rosella, VkCommon common, Renderer renderer) {
        this.shaderManager = new ShaderManager(rosella);
        this.textureManager = new TextureManager(common);
        this.pipelineManager = new PipelineManager(common, renderer);
        this.renderer = renderer;
    }

    @Override
    public void rebuildCmdBuffers(RenderPass pass, Rosella rosella, Renderer renderer) {

    }

    @Override
    public Renderable addObject(Renderable renderable) {
        return renderable;
    }

    @Override
    public Material registerMaterial(Material material) {
        return material;
    }

    @Override
    public ShaderProgram addShader(RawShaderProgram program) {
        return shaderManager.getOrCreateShader(program);
    }

    @Override
    public void submitMaterials() {
        for (Material material : unprocessedMaterials) {
            if (material.getShader().getRaw().getDescriptorSetLayout() == 0L) {
                material.getShader().getRaw().createDescriptorSetLayout();
                material.pipeline = pipelineManager.getPipeline(material, renderer);
                materials.add(material);
            }
        }
        unprocessedMaterials.clear();
    }

    @Override
    public void free(Rosella rosella) {
        for (Material material : materials) {
            material.getShader().free();
        }
        materials.clear();

        shaderManager.free();
    }
}
