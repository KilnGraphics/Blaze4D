package me.hydos.rosella.render.material;

import me.hydos.rosella.render.Topology;
import me.hydos.rosella.render.material.state.StateInfo;
import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.swapchain.RenderPass;
import me.hydos.rosella.render.vertex.VertexFormat;

public record PipelineCreateInfo(RenderPass renderPass, long descriptorSetLayout,
                                 int polygonMode, ShaderProgram shader,
                                 Topology topology,
                                 VertexFormat vertexFormat,
                                 StateInfo stateInfo) {
}
