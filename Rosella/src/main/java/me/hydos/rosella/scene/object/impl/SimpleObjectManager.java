package me.hydos.rosella.scene.object.impl;

import it.unimi.dsi.fastutil.objects.ObjectArrayList;
import me.hydos.rosella.Rosella;
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

import java.util.*;

/**
 * Just a basic object manager
 */
public class SimpleObjectManager implements ObjectManager {

    public Renderer renderer;
    private final VkCommon common;
    private final Rosella rosella;
    public final ShaderManager shaderManager;
    public final TextureManager textureManager;
    public PipelineManager pipelineManager;
    public final List<Renderable> renderObjects = new ObjectArrayList<>(1024);

    public final List<Material> materials = new ArrayList<>();
    public final List<Material> unprocessedMaterials = new ArrayList<>();

    public SimpleObjectManager(Rosella rosella, VkCommon common) {
        this.shaderManager = new ShaderManager(rosella);
        this.textureManager = new TextureManager(common);
        this.rosella = rosella;
        this.common = common;
    }

    @Override
    public void rebuildCmdBuffers(RenderPass pass, Rosella rosella, Renderer renderer) {
        // TODO: move to here
    }

    @Override
    public Renderable addObject(Renderable obj) {
        obj.onAddedToScene(rosella);
        renderObjects.add(obj);
        return obj;
    }

    @Override
    public Material registerMaterial(Material material) {
        unprocessedMaterials.add(material);
        return material;
    }

    @Override
    public ShaderProgram addShader(RawShaderProgram program) {
        return shaderManager.getOrCreateShader(program);
    }

    @Override
    public void submitMaterials() {
        for (Material material : unprocessedMaterials) {
            if (material.getShaderProgram().getRaw().getDescriptorSetLayout() == 0L) {
                material.getShaderProgram().getRaw().createDescriptorSetLayout();
            }
            material.setPipeline(pipelineManager.getOrCreatePipeline(material, renderer));
            materials.add(material);
        }
        unprocessedMaterials.clear();
    }

    @Override
    public void free() {
        // TODO: why? this should just get picked up by the gc i think
        materials.clear();

        shaderManager.free();
    }

    @Override
    public void postInit(Renderer renderer) {
        this.renderer = renderer;
        this.pipelineManager = new PipelineManager(common, renderer);
    }
}
