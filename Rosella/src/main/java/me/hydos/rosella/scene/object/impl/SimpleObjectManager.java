package me.hydos.rosella.scene.object.impl;

import it.unimi.dsi.fastutil.objects.ObjectArrayList;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.render.pipeline.PipelineManager;
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
    public final List<Renderable> renderObjects = new ObjectArrayList<>();

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
    public ShaderProgram addShader(RawShaderProgram program) {
        return shaderManager.getOrCreateShader(program);
    }

    @Override
    public void free() {
        shaderManager.free();
    }

    @Override
    public void postInit(Renderer renderer) {
        this.renderer = renderer;
        this.pipelineManager = new PipelineManager(common, renderer);
    }
}
