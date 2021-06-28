package me.hydos.rosella.render.material

import me.hydos.rosella.Rosella
import me.hydos.rosella.render.Topology
import me.hydos.rosella.render.device.Device
import me.hydos.rosella.render.renderer.Renderer
import me.hydos.rosella.render.shader.ShaderProgram
import me.hydos.rosella.render.swapchain.RenderPass
import me.hydos.rosella.render.swapchain.SwapChain
import me.hydos.rosella.render.util.ok
import me.hydos.rosella.render.vertex.VertexFormat
import org.lwjgl.system.MemoryStack
import org.lwjgl.vulkan.*
import java.nio.ByteBuffer
import java.nio.LongBuffer

class PipelineManager(var swapchain: SwapChain, val device: Device) {

	private val pipelines = HashMap<PipelineCreateInfo, PipelineInfo>()

	private fun getPipeline(createInfo: PipelineCreateInfo): PipelineInfo {
		if (!pipelines.containsKey(createInfo)) {
			pipelines[createInfo] = createPipeline(
				device,
				swapchain,
				createInfo.renderPass,
				createInfo.descriptorSetLayout,
				createInfo.polygonMode,
				createInfo.shader,
				createInfo.topology,
				createInfo.vertexFormat,
				createInfo.useBlend
			)
		}
		return pipelines[createInfo]!!
	}

	fun getPipeline(material: Material, renderer: Renderer, rosella: Rosella): PipelineInfo {
		val createInfo = PipelineCreateInfo(
			renderer.renderPass,
			material.shader.raw.descriptorSetLayout,
			rosella.polygonMode,
			material.shader,
			material.topology,
			material.vertexFormat,
			material.useBlend
		)
		return getPipeline(createInfo)
	}

	fun invalidatePipelines(swapchain: SwapChain, rosella: Rosella) {
		for (pipeline in pipelines.values) {
			VK10.vkDestroyPipeline(device.device, pipeline.graphicsPipeline, null)
			VK10.vkDestroyPipelineLayout(device.device, pipeline.pipelineLayout, null)
		}

		pipelines.clear()
		rosella.renderer.rebuildCommandBuffers(rosella.renderer.renderPass, rosella)
		this.swapchain = swapchain
	}

	fun isValidPipeline(pipeline: PipelineInfo): Boolean {
		return pipelines.values.contains(pipeline)
	}

	/**
	 * Creates a new pipeline
	 */
	private fun createPipeline(
		device: Device,
		swapChain: SwapChain,
		renderPass: RenderPass,
		descriptorSetLayout: Long,
		polygonMode: Int,
		shader: ShaderProgram,
		topology: Topology,
		vertexFormat: VertexFormat,
		useBlend: Boolean
	): PipelineInfo {
		var pipelineLayout: Long
		var graphicsPipeline: Long

		MemoryStack.stackPush().use {
			val vertShaderModule = shader.getVertShaderModule()
			val fragShaderModule = shader.getFragShaderModule()

			val entryPoint: ByteBuffer = it.UTF8("main")
			val shaderStages = VkPipelineShaderStageCreateInfo.callocStack(2, it)

			shaderStages[0]
				.sType(VK10.VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO)
				.stage(VK10.VK_SHADER_STAGE_VERTEX_BIT)
				.module(vertShaderModule)
				.pName(entryPoint)

			shaderStages[1]
				.sType(VK10.VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO)
				.stage(VK10.VK_SHADER_STAGE_FRAGMENT_BIT)
				.module(fragShaderModule)
				.pName(entryPoint)

			/**
			 * Vertex
			 */
			val vertexInputInfo: VkPipelineVertexInputStateCreateInfo =
				VkPipelineVertexInputStateCreateInfo.callocStack(it)
					.sType(VK10.VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO)
					.pVertexBindingDescriptions(vertexFormat.vkBindings)
					.pVertexAttributeDescriptions(vertexFormat.vkAttributes)

			/**
			 * Assembly
			 */
			val inputAssembly: VkPipelineInputAssemblyStateCreateInfo =
				VkPipelineInputAssemblyStateCreateInfo.callocStack(it)
					.sType(VK10.VK_STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO)
					.topology(topology.vkType)
					.primitiveRestartEnable(false)

			/**
			 * Viewport
			 */
			val viewport = VkViewport.callocStack(1, it)
				.x(0.0f)
				.y(0.0f)
				.width(swapChain.swapChainExtent.width().toFloat())
				.height(swapChain.swapChainExtent.height().toFloat())
				.minDepth(0.0f)
				.maxDepth(1.0f)

			/**
			 * Scissor
			 */
			val scissor = VkRect2D.callocStack(1, it)
				.offset(VkOffset2D.callocStack(it).set(0, 0))
				.extent(swapChain.swapChainExtent)

			/**
			 * Viewport State
			 */
			val viewportState: VkPipelineViewportStateCreateInfo = VkPipelineViewportStateCreateInfo.callocStack(it)
				.sType(VK10.VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO)
				.pViewports(viewport)
				.pScissors(scissor)

			/**
			 * Rasterisation
			 */
			val rasterizer: VkPipelineRasterizationStateCreateInfo =
				VkPipelineRasterizationStateCreateInfo.callocStack(it)
					.sType(VK10.VK_STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO)
					.depthClampEnable(false)
					.rasterizerDiscardEnable(false)
					.polygonMode(polygonMode)
					.lineWidth(1.0f)
					.cullMode(VK10.VK_CULL_MODE_BACK_BIT)
					.frontFace(VK10.VK_FRONT_FACE_COUNTER_CLOCKWISE)
					.depthBiasEnable(false)

			/**
			 * Multisampling
			 */
			val multisampling: VkPipelineMultisampleStateCreateInfo =
				VkPipelineMultisampleStateCreateInfo.callocStack(it)
					.sType(VK10.VK_STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO)
					.sampleShadingEnable(false)
					.rasterizationSamples(VK10.VK_SAMPLE_COUNT_1_BIT)

			val depthStencil: VkPipelineDepthStencilStateCreateInfo =
				VkPipelineDepthStencilStateCreateInfo.callocStack(it)
					.sType(VK10.VK_STRUCTURE_TYPE_PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO)
					.depthTestEnable(true)
					.depthWriteEnable(true)
					.depthCompareOp(VK10.VK_COMPARE_OP_LESS)
					.depthBoundsTestEnable(false)
					.minDepthBounds(0.0f)
					.maxDepthBounds(1.0f)
					.stencilTestEnable(false)

			/**
			 * Colour Blending
			 */
			val colourBlendAttachment = VkPipelineColorBlendAttachmentState.callocStack(1, it)
				.colorWriteMask(VK10.VK_COLOR_COMPONENT_R_BIT or VK10.VK_COLOR_COMPONENT_G_BIT or VK10.VK_COLOR_COMPONENT_B_BIT or VK10.VK_COLOR_COMPONENT_A_BIT)
				.blendEnable(useBlend)
				.srcColorBlendFactor(VK10.VK_BLEND_FACTOR_SRC_ALPHA)
				.dstColorBlendFactor(VK10.VK_BLEND_FACTOR_ONE_MINUS_SRC_ALPHA)
				.colorBlendOp(VK10.VK_BLEND_OP_ADD)

				.srcAlphaBlendFactor(VK10.VK_BLEND_FACTOR_ZERO)
				.dstAlphaBlendFactor(VK10.VK_BLEND_FACTOR_ONE_MINUS_SRC_ALPHA)
				.alphaBlendOp(VK10.VK_BLEND_OP_SUBTRACT)

			val colourBlending: VkPipelineColorBlendStateCreateInfo =
				VkPipelineColorBlendStateCreateInfo.callocStack(it)
					.sType(VK10.VK_STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO)
					.logicOpEnable(false)
					.logicOp(VK10.VK_LOGIC_OP_COPY)
					.pAttachments(colourBlendAttachment)
					.blendConstants(it.floats(0.0f, 0.0f, 0.0f, 0.0f))

/*			*/
			/**
			 * Create Push Constants
			 *//*
			val pushConstantRange = VkPushConstantRange.callocStack(1, it)
				.stageFlags(VK_SHADER_STAGE_VERTEX_BIT)
				.offset(0)
				.size(sizeof(Vector3f::class))*/

			/**
			 * Pipeline Layout Creation
			 */
			val pipelineLayoutInfo: VkPipelineLayoutCreateInfo = VkPipelineLayoutCreateInfo.callocStack(it)
				.sType(VK10.VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO)
				.pSetLayouts(it.longs(descriptorSetLayout))
//				.pPushConstantRanges(pushConstantRange)

			val pPipelineLayout = it.longs(VK10.VK_NULL_HANDLE)
			VK10.vkCreatePipelineLayout(device.device, pipelineLayoutInfo, null, pPipelineLayout).ok()
			pipelineLayout = pPipelineLayout[0]

			/**
			 * Pipeline Creation
			 */
			val pipelineInfo = VkGraphicsPipelineCreateInfo.callocStack(1, it)
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
				.renderPass(renderPass.renderPass)
				.subpass(0)
				.basePipelineHandle(VK10.VK_NULL_HANDLE)
				.basePipelineIndex(-1)

			val pGraphicsPipeline: LongBuffer = it.mallocLong(1)
			VK10.vkCreateGraphicsPipelines(device.device, VK10.VK_NULL_HANDLE, pipelineInfo, null, pGraphicsPipeline)
				.ok()
			graphicsPipeline = pGraphicsPipeline[0]

			VK10.vkDestroyShaderModule(device.device, vertShaderModule, null)
			VK10.vkDestroyShaderModule(device.device, fragShaderModule, null)

			return PipelineInfo(pipelineLayout, graphicsPipeline)
		}
	}
}