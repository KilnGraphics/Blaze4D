package me.hydos.rosella.render.pipeline;

import me.hydos.rosella.render.PolygonMode;
import me.hydos.rosella.render.Topology;
import me.hydos.rosella.render.pipeline.state.StateInfo;
import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.swapchain.RenderPass;
import me.hydos.rosella.render.vertex.VertexFormat;

public record PipelineCreateInfo(RenderPass renderPass,
                                 ShaderProgram shaderProgram,
                                 Topology topology,
                                 PolygonMode polygonMode,
                                 VertexFormat vertexFormat,
                                 StateInfo stateInfo) {
}
