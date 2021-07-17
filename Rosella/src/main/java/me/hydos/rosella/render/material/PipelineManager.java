package me.hydos.rosella.render.material;

import java.nio.ByteBuffer;
import java.nio.LongBuffer;
import java.util.HashMap;
import java.util.Map;

import me.hydos.rosella.Rosella;
import me.hydos.rosella.device.VulkanDevice;
import me.hydos.rosella.render.Topology;
import me.hydos.rosella.render.material.state.StateInfo;
import me.hydos.rosella.render.renderer.Renderer;
import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.swapchain.RenderPass;
import me.hydos.rosella.render.swapchain.Swapchain;
import me.hydos.rosella.render.vertex.VertexFormat;
import me.hydos.rosella.vkobjects.VkCommon;
import org.jetbrains.annotations.NotNull;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.vulkan.VK10;
import org.lwjgl.vulkan.VkGraphicsPipelineCreateInfo;
import org.lwjgl.vulkan.VkOffset2D;
import org.lwjgl.vulkan.VkPipelineColorBlendAttachmentState;
import org.lwjgl.vulkan.VkPipelineColorBlendStateCreateInfo;
import org.lwjgl.vulkan.VkPipelineDepthStencilStateCreateInfo;
import org.lwjgl.vulkan.VkPipelineInputAssemblyStateCreateInfo;
import org.lwjgl.vulkan.VkPipelineLayoutCreateInfo;
import org.lwjgl.vulkan.VkPipelineMultisampleStateCreateInfo;
import org.lwjgl.vulkan.VkPipelineRasterizationStateCreateInfo;
import org.lwjgl.vulkan.VkPipelineShaderStageCreateInfo;
import org.lwjgl.vulkan.VkPipelineVertexInputStateCreateInfo;
import org.lwjgl.vulkan.VkPipelineViewportStateCreateInfo;
import org.lwjgl.vulkan.VkRect2D;
import org.lwjgl.vulkan.VkViewport;

import static me.hydos.rosella.util.VkUtils.ok;

public class PipelineManager {
    private final VkCommon common;
    private final Renderer renderer;

    private final Map<PipelineCreateInfo, PipelineInfo> pipelines = new HashMap<>();

    public PipelineManager(VkCommon common, Renderer renderer) {
        this.common = common;
        this.renderer = renderer;
    }

    private PipelineInfo getPipeline(PipelineCreateInfo createInfo) {
        return pipelines.computeIfAbsent(createInfo, _createInfo -> createPipeline(
                common.device,
                renderer.swapchain,
                createInfo.renderPass(),
                createInfo.descriptorSetLayout(),
                createInfo.polygonMode(),
                createInfo.shader(),
                createInfo.topology(),
                createInfo.vertexFormat(),
                createInfo.stateInfo()
        ));
    }

    @NotNull
    public PipelineInfo getPipeline(Material material, Renderer renderer) {
        PipelineCreateInfo createInfo = new PipelineCreateInfo(
                renderer.renderPass,
                material.shader.getRaw().getDescriptorSetLayout(),
                Rosella.POLYGON_MODE,
                material.shader,
                material.topology,
                material.vertexFormat,
                material.stateInfo
        );

        return getPipeline(createInfo);
    }

    public void invalidatePipelines(VkCommon common) {
        pipelines.forEach((pipelineCreateInfo, pipelineInfo) -> pipelineInfo.free(common.device, common.memory));
        pipelines.clear();
    }

    /**
     * Creates a new pipeline
     */
    private PipelineInfo createPipeline(VulkanDevice device, Swapchain swapchain, RenderPass renderPass, Long descriptorSetLayout, int polygonMode, ShaderProgram shader, Topology topology, VertexFormat vertexFormat, StateInfo stateInfo) {
        long pipelineLayout;
        long graphicsPipeline;

        try (MemoryStack stack = MemoryStack.stackPush()) {
            long vertShaderModule = shader.getVertShaderModule();
            long fragShaderModule = shader.getFragShaderModule();

            ByteBuffer entryPoint = stack.UTF8("main");
            VkPipelineShaderStageCreateInfo.Buffer shaderStages = VkPipelineShaderStageCreateInfo.callocStack(2, stack);

            shaderStages.get(0)
                    .sType(VK10.VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO)
                    .stage(VK10.VK_SHADER_STAGE_VERTEX_BIT)
                    .module(vertShaderModule)
                    .pName(entryPoint);

            shaderStages.get(1)
                    .sType(VK10.VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO)
                    .stage(VK10.VK_SHADER_STAGE_FRAGMENT_BIT)
                    .module(fragShaderModule)
                    .pName(entryPoint);

            VkPipelineVertexInputStateCreateInfo vertexInputInfo = vertexFormat.getPipelineVertexInputStateCreateInfo(stack);

            VkPipelineInputAssemblyStateCreateInfo inputAssembly = getPipelineInputAssemblyStateCreateInfo(topology, stack);

            VkViewport.Buffer viewport = getViewport(swapchain, stack);

            VkRect2D.Buffer scissor = stateInfo.isScissorEnabled() ? stateInfo.getExtent(stack) : getDefaultScissor(swapchain, stack);

            VkPipelineViewportStateCreateInfo viewportState = VkPipelineViewportStateCreateInfo.callocStack(stack)
                    .sType(VK10.VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO)
                    .pViewports(viewport)
                    .pScissors(scissor);

            VkPipelineRasterizationStateCreateInfo rasterizer = stateInfo.getRasterizationStateCreateInfo(polygonMode, stack);

            VkPipelineMultisampleStateCreateInfo multisampling = VkPipelineMultisampleStateCreateInfo.callocStack(stack)
                            .sType(VK10.VK_STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO)
                            .sampleShadingEnable(false)
                            .rasterizationSamples(VK10.VK_SAMPLE_COUNT_1_BIT);

            VkPipelineDepthStencilStateCreateInfo depthStencil = stateInfo.getPipelineDepthStencilStateCreateInfo(stack);

            VkPipelineColorBlendAttachmentState.Buffer colourBlendAttachment = stateInfo.getPipelineColorBlendAttachmentStates(stack);

            VkPipelineColorBlendStateCreateInfo colourBlending = stateInfo.getPipelineColorBlendStateCreateInfo(stack, colourBlendAttachment);

            VkPipelineLayoutCreateInfo pipelineLayoutInfo = VkPipelineLayoutCreateInfo.callocStack(stack)
                    .sType(VK10.VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO)
                    .pSetLayouts(stack.longs(descriptorSetLayout));

            LongBuffer pPipelineLayout = stack.longs(VK10.VK_NULL_HANDLE);
            ok(VK10.vkCreatePipelineLayout(device.rawDevice, pipelineLayoutInfo, null, pPipelineLayout));
            pipelineLayout = pPipelineLayout.get(0);

            VkGraphicsPipelineCreateInfo.Buffer pipelineInfo = VkGraphicsPipelineCreateInfo.callocStack(1, stack)
                    .sType(VK10.VK_STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO)
                    .pStages(shaderStages)
                    .pVertexInputState(vertexInputInfo)
                    .pInputAssemblyState(inputAssembly)
                    .pViewportState(viewportState)
                    .pRasterizationState(rasterizer)
                    .pMultisampleState(multisampling)
                    .pDepthStencilState(depthStencil)
                    .pColorBlendState(colourBlending)
                    .layout(pipelineLayout)
                    .renderPass(renderPass.getRenderPass())
                    .subpass(0)
                    .basePipelineHandle(VK10.VK_NULL_HANDLE)
                    .basePipelineIndex(-1);

            LongBuffer pGraphicsPipeline = stack.mallocLong(1);
            ok(VK10.vkCreateGraphicsPipelines(device.rawDevice, VK10.VK_NULL_HANDLE, pipelineInfo, null, pGraphicsPipeline));
            graphicsPipeline = pGraphicsPipeline.get(0);

            VK10.vkDestroyShaderModule(device.rawDevice, vertShaderModule, null);
            VK10.vkDestroyShaderModule(device.rawDevice, fragShaderModule, null);

            return new PipelineInfo(pipelineLayout, graphicsPipeline);
        }
    }

    // TODO: Fix once Topology is rewritten in java
    private VkPipelineInputAssemblyStateCreateInfo getPipelineInputAssemblyStateCreateInfo(Topology topology, MemoryStack stack) {
        return VkPipelineInputAssemblyStateCreateInfo.callocStack(stack)
                .sType(VK10.VK_STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO)
                .topology(topology.vkType)
                .primitiveRestartEnable(false);
    }

    // TODO: Fix once Swapchain is rewritten in java
    @NotNull
    private VkRect2D.Buffer getDefaultScissor(Swapchain swapchain, MemoryStack stack) {
        return VkRect2D.callocStack(1, stack)
                .offset(VkOffset2D.callocStack(stack).set(0, 0))
                .extent(swapchain.getSwapChainExtent());
    }

    // TODO: Fix once Swapchain is rewritten in java
    @NotNull
    private VkViewport.Buffer getViewport(Swapchain swapchain, MemoryStack stack) {
        return VkViewport.callocStack(1, stack)
                .x(0.0f)
                .y(0.0f)
                .width(swapchain.getSwapChainExtent().width())
                .height(swapchain.getSwapChainExtent().height())
                .minDepth(0.0f)
                .maxDepth(1.0f);
    }
}
