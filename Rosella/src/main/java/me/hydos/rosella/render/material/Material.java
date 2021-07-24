package me.hydos.rosella.render.material;

import me.hydos.rosella.render.Topology;
import me.hydos.rosella.render.material.state.StateInfo;
import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.vertex.VertexFormat;

import java.util.Objects;

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

    protected PipelineInfo pipeline;

    public Material(ShaderProgram shaderProgram, Topology topology, VertexFormat vertexFormat, StateInfo stateInfo) {
        this.shaderProgram = shaderProgram;
        this.topology = topology;
        this.vertexFormat = vertexFormat;
        this.stateInfo = stateInfo;
    }

    public ShaderProgram getShaderProgram() {
        return shaderProgram;
    }

    public PipelineInfo getPipeline() {
        return pipeline;
    }

    public void setPipeline(PipelineInfo pipeline) {
        this.pipeline = pipeline;
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        Material material = (Material) o;
        return shaderProgram.equals(material.shaderProgram) && topology == material.topology && vertexFormat.equals(material.vertexFormat) && stateInfo.equals(material.stateInfo);
    }

    @Override
    public int hashCode() {
        return Objects.hash(shaderProgram, topology, vertexFormat, stateInfo);
    }
}

