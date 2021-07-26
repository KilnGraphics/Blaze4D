package me.hydos.blaze4d.api.vertex;

import me.hydos.blaze4d.api.GlobalRenderSystem;
import me.hydos.blaze4d.api.Materials;
import me.hydos.blaze4d.api.shader.MinecraftShaderProgram;
import me.hydos.blaze4d.api.shader.MinecraftUbo;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.device.VulkanDevice;
import me.hydos.rosella.memory.Memory;
import me.hydos.rosella.render.Topology;
import me.hydos.rosella.render.info.InstanceInfo;
import me.hydos.rosella.render.info.RenderInfo;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.material.state.StateInfo;
import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.texture.Texture;
import me.hydos.rosella.render.vertex.VertexFormat;
import me.hydos.rosella.scene.object.Renderable;
import org.joml.Matrix4f;
import org.joml.Vector3f;

import java.util.Objects;
import java.util.concurrent.Future;

public class ConsumerRenderObject implements Renderable {

    // Render Implementation Fields
    private final Future<RenderInfo> renderInfo;
    private final InstanceInfo instanceInfo;

    public ConsumerRenderObject(
            Future<RenderInfo> renderInfo,
            VertexFormat format,
            Topology topology,
            ShaderProgram shader,
            Texture[] textures,
            StateInfo stateInfo,
            Rosella rosella) {

        this.renderInfo = renderInfo;
        Material material = Materials.getOrCreateMaterial(topology, shader, format, stateInfo);
        this.instanceInfo = new InstanceInfo(((MinecraftShaderProgram) shader.getRaw()).createMinecraftUbo(rosella.common.memory, material, GlobalRenderSystem.getShaderUbo()), textures, material);
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
