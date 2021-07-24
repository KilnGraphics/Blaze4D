package me.hydos.rosella.render.material;

import me.hydos.rosella.render.Topology;
import me.hydos.rosella.render.material.state.StateInfo;
import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.vertex.VertexFormat;

/**
 * A Material is like texture information, normal information, and all of those things which give an object character wrapped into one class.
 * similar to how unity material's works
 * guaranteed to change in the future
 */
public class Material {

    protected final ShaderProgram shaderProgram;
    protected final Topology topology;
    protected final VertexFormat vertexFormat;
    protected final StateInfo stateInfo;

    public Material(ShaderProgram shaderProgram, Topology topology, VertexFormat vertexFormat, StateInfo stateInfo) {
        this.shaderProgram = shaderProgram;
        this.topology = topology;
        this.vertexFormat = vertexFormat;
        this.stateInfo = stateInfo;
    }

    protected PipelineInfo pipeline;

    public ShaderProgram getShaderProgram() {
        return shaderProgram;
    }

    public PipelineInfo getPipeline() {
        return pipeline;
    }

    public void setPipeline(PipelineInfo pipeline) {
        this.pipeline = pipeline;
    }
}

