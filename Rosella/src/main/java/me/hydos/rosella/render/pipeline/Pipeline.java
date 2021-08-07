package me.hydos.rosella.render.pipeline;

import me.hydos.rosella.device.LegacyVulkanDevice;
import me.hydos.rosella.memory.Memory;
import me.hydos.rosella.memory.MemoryCloseable;
import me.hydos.rosella.render.PolygonMode;
import me.hydos.rosella.render.Topology;
import me.hydos.rosella.render.pipeline.state.StateInfo;
import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.swapchain.RenderPass;
import me.hydos.rosella.render.vertex.VertexFormat;

import java.util.Objects;

public class Pipeline implements MemoryCloseable {

    private final RenderPass renderPass;
    private final ShaderProgram shaderProgram;
    private final Topology topology;
    private final PolygonMode polygonMode;
    private final VertexFormat vertexFormat;
    private final StateInfo stateInfo;

    private long pipelineLayout;
    private long graphicsPipeline;

    public Pipeline(RenderPass renderPass,
                    ShaderProgram shaderProgram,
                    Topology topology,
                    PolygonMode polygonMode,
                    VertexFormat vertexFormat,
                    StateInfo stateInfo) {

        this.renderPass = renderPass;
        this.shaderProgram = shaderProgram;
        this.topology = topology;
        this.polygonMode = polygonMode;
        this.vertexFormat = vertexFormat;
        this.stateInfo = stateInfo;
    }

    public RenderPass getRenderPass() {
        return renderPass;
    }

    public ShaderProgram getShaderProgram() {
        return shaderProgram;
    }

    public Topology getTopology() {
        return topology;
    }

    public PolygonMode getPolygonMode() {
        return polygonMode;
    }

    public VertexFormat getVertexFormat() {
        return vertexFormat;
    }

    public StateInfo getStateInfo() {
        return stateInfo;
    }

    public long getPipelineLayout() {
        return pipelineLayout;
    }

    public long getGraphicsPipeline() {
        return graphicsPipeline;
    }

    void setRawInfo(long pipelineLayout, long graphicsPipeline) {
        this.pipelineLayout = pipelineLayout;
        this.graphicsPipeline = graphicsPipeline;
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        Pipeline pipeline = (Pipeline) o;
        return renderPass.equals(pipeline.renderPass) && shaderProgram.equals(pipeline.shaderProgram) && topology == pipeline.topology && polygonMode == pipeline.polygonMode && vertexFormat.equals(pipeline.vertexFormat) && stateInfo.equals(pipeline.stateInfo);
    }

    @Override
    public int hashCode() {
        return Objects.hash(renderPass, shaderProgram, topology, polygonMode, vertexFormat, stateInfo);
    }

    @Override
    public void free(LegacyVulkanDevice device, Memory memory) {
        memory.freePipeline(this);
    }
}
