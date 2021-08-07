package me.hydos.rosella.scene.object;

import me.hydos.rosella.Rosella;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.pipeline.Pipeline;
import me.hydos.rosella.render.renderer.Renderer;
import me.hydos.rosella.render.shader.RawShaderProgram;
import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.swapchain.RenderPass;
import me.hydos.rosella.render.texture.TextureMap;

/**
 * Allows for multiple ways for the engine to handle objects.
 */
public interface ObjectManager {

    /**
     * Called when the renderer wants to rebuild command buffers
     *
     * @param pass    the current render pass
     * @param rosella the instance of the engine.
     */
    void rebuildCmdBuffers(RenderPass pass, Rosella rosella, Renderer renderer);

    /**
     * adds an object into the current scene.
     *
     * @param renderable the material to add to the scene
     */
    Renderable addObject(Renderable renderable);

    /**
     * registers a {@link RawShaderProgram} into the engine.
     *
     * @param program the program to register
     */
    ShaderProgram addShader(RawShaderProgram program);

    /**
     * Called when the engine is exiting.
     */
    void free();

    /**
     * Called after an instance of the renderer is created
     *
     * @param renderer the renderer
     */
    void postInit(Renderer renderer);
}

