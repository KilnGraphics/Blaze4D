package me.hydos.blaze4d.api.vertex;

import me.hydos.blaze4d.api.shader.MinecraftShaderProgram;
import graphics.kiln.rosella.Rosella;
import graphics.kiln.rosella.device.VulkanDevice;
import graphics.kiln.rosella.memory.Memory;
import graphics.kiln.rosella.render.PolygonMode;
import graphics.kiln.rosella.render.Topology;
import graphics.kiln.rosella.render.info.InstanceInfo;
import graphics.kiln.rosella.render.info.RenderInfo;
import graphics.kiln.rosella.render.material.Material;
import graphics.kiln.rosella.render.pipeline.Pipeline;
import graphics.kiln.rosella.render.pipeline.state.StateInfo;
import graphics.kiln.rosella.render.shader.ShaderProgram;
import graphics.kiln.rosella.render.texture.TextureMap;
import graphics.kiln.rosella.render.vertex.VertexFormat;
import graphics.kiln.rosella.scene.object.Renderable;
import graphics.kiln.rosella.scene.object.impl.SimpleObjectManager;

import java.nio.ByteBuffer;
import java.util.Objects;
import java.util.concurrent.Future;

public class ConsumerRenderObject implements Renderable {

    // Render Implementation Fields
    private final Future<RenderInfo> renderInfo;
    private final InstanceInfo instanceInfo;

    public ConsumerRenderObject(
            Future<RenderInfo> renderInfo,
            ShaderProgram shaderProgram,
            Topology topology,
            VertexFormat vertexFormat,
            StateInfo stateInfo,
            TextureMap textures,
            ByteBuffer rawUboData,
            Rosella rosella) {

        this.renderInfo = renderInfo;
        Material material = new Material(
                rosella.common.pipelineManager.registerPipeline(
                        new Pipeline(
                                rosella.renderer.mainRenderPass, // TODO: make render passes less jank, more info in rosella comments
                                shaderProgram,
                                topology,
                                vertexFormat,
                                stateInfo
                        )
                ),
                textures
        );
        this.instanceInfo = new InstanceInfo(((MinecraftShaderProgram) shaderProgram.getRaw()).createMinecraftUbo(rosella.common.memory, shaderProgram.getRaw().getDescriptorPool(), rawUboData), material);
    }

    //======================
    // Render Implementation
    //======================

    @Override
    public void onAddedToScene(Rosella rosella) {
        // WE DO NOT NEED TO HARD REBUILD HERE
    }

    @Override
    public void free(VulkanDevice device, Memory memory) {
        instanceInfo.free(device, memory);
        // we don't want to free the RenderInfo here because they can exist across frames
    }

    @Override
    public void rebuild(Rosella rosella) {
        instanceInfo.rebuild(rosella);
    }

    @Override
    public void hardRebuild(Rosella rosella) {
        instanceInfo.hardRebuild(rosella);
    }

    @Override
    public InstanceInfo getInstanceInfo() {
        return instanceInfo;
    }

    @Override
    public Future<RenderInfo> getRenderInfo() {
        return renderInfo;
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        ConsumerRenderObject that = (ConsumerRenderObject) o;
        return renderInfo.equals(that.renderInfo) && instanceInfo.equals(that.instanceInfo);
    }

    @Override
    public int hashCode() {
        return Objects.hash(renderInfo, instanceInfo);
    }
}
