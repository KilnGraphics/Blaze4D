package me.hydos.rosella.render.material

import me.hydos.rosella.Rosella
import me.hydos.rosella.device.VulkanDevice
import me.hydos.rosella.render.Topology
import me.hydos.rosella.render.material.state.StateInfo
import me.hydos.rosella.render.renderer.Renderer
import me.hydos.rosella.render.shader.ShaderProgram
import me.hydos.rosella.render.swapchain.RenderPass
import me.hydos.rosella.render.swapchain.Swapchain
import me.hydos.rosella.render.util.ok
import me.hydos.rosella.render.vertex.VertexFormat
import me.hydos.rosella.scene.`object`.impl.SimpleObjectManager
import me.hydos.rosella.vkobjects.VkCommon
import org.lwjgl.system.MemoryStack
import org.lwjgl.vulkan.*
import java.nio.ByteBuffer
import java.nio.LongBuffer

class PipelineManager(var common: VkCommon, val renderer: Renderer) {

	private val pipelines = HashMap<PipelineCreateInfo, PipelineInfo>()

	private fun getPipeline(createInfo: PipelineCreateInfo): PipelineInfo {
		if (!pipelines.containsKey(createInfo)) {
			pipelines[createInfo] = createPipeline(
				common.device,
				renderer.swapchain,
				createInfo.renderPass,
				createInfo.descriptorSetLayout,
				createInfo.polygonMode,
				createInfo.shader,
				createInfo.topology,
				createInfo.vertexFormat,
				createInfo.stateInfo
			)
		}
		return pipelines[createInfo]!!
	}

	fun getPipeline(material: Material, renderer: Renderer): PipelineInfo {
		val createInfo = PipelineCreateInfo(
			renderer.renderPass,
			material.shader!!.raw.descriptorSetLayout,
			Rosella.POLYGON_MODE,
			material.shader!!,
			material.topology,
			material.vertexFormat,
			material.stateInfo
		)
		return getPipeline(createInfo)
	}

	fun invalidatePipelines(swapchain: Swapchain, rosella: Rosella) {
		for (pipeline in pipelines.values) {
			VK10.vkDestroyPipeline(rosella.common.device.rawDevice, pipeline.graphicsPipeline, null)
			VK10.vkDestroyPipelineLayout(rosella.common.device.rawDevice, pipeline.pipelineLayout, null)
		}

		pipelines.clear()
		rosella.renderer.rebuildCommandBuffers(
			rosella.renderer.renderPass,
			rosella.objectManager as SimpleObjectManager
		)
	}

	fun isValidPipeline(pipeline: PipelineInfo): Boolean {
		return pipelines.values.contains(pipeline)
	}

	/**
	 * Creates a new pipeline
	 */
	private fun createPipeline(
		device: VulkanDevice,
		swapchain: Swapchain,
		renderPass: RenderPass,
		descriptorSetLayout: Long,
		polygonMode: Int,
		shader: ShaderProgram,
		topology: Topology,
		vertexFormat: VertexFormat,
		stateInfo: StateInfo
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
				.width(swapchain.swapChainExtent.width().toFloat())
				.height(swapchain.swapChainExtent.height().toFloat())
				.minDepth(0.0f)
				.maxDepth(1.0f)

			/**
			 * Viewport State
			 */
			val viewportState: VkPipelineViewportStateCreateInfo = VkPipelineViewportStateCreateInfo.callocStack(it)
				.sType(VK10.VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO)
				.pViewports(viewport)

			/**
			 * Scissor
			 */
			// TODO: make sure we can ignore setting pScissors safely
			if (stateInfo.scissorEnabled) {
				val scissor = VkRect2D.callocStack(1, it)
					.offset(VkOffset2D.callocStack(it).set(stateInfo.scissorX, stateInfo.scissorY))
					.extent(VkExtent2D.callocStack(it).set(stateInfo.scissorWidth, stateInfo.scissorHeight))

				viewportState.pScissors(scissor)
			}

			/**
			 * Rasterisation
			 */
			val rasterizer: VkPipelineRasterizationStateCreateInfo =
				VkPipelineRasterizationStateCreateInfo.callocStack(it)
					.sType(VK10.VK_STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO)
					.depthClampEnable(false)
					.rasterizerDiscardEnable(false)
					.polygonMode(polygonMode)
					.lineWidth(stateInfo.lineWidth)
					.cullMode(if (stateInfo.cullEnabled) VK10.VK_CULL_MODE_BACK_BIT else VK10.VK_CULL_MODE_NONE)
					.frontFace(VK10.VK_FRONT_FACE_COUNTER_CLOCKWISE) // TODO: try messing with this
					.depthBiasEnable(false) // TODO: possibly causes flickering

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
					.depthTestEnable(stateInfo.depthTestEnabled)
					.depthWriteEnable(stateInfo.depthMask)
					.depthCompareOp(stateInfo.depthCompareOp)
					.depthBoundsTestEnable(false)
					.minDepthBounds(0.0f)
					.maxDepthBounds(1.0f)
					.stencilTestEnable(stateInfo.stencilEnabled) // TODO: fix stencil settings

			/**
			 * Colour Blending
			 */
			// TODO: use minecraft's blending info from the shaders
			val colourBlendAttachment = VkPipelineColorBlendAttachmentState.callocStack(1, it)
				.colorWriteMask(stateInfo.colorMask)
				.blendEnable(stateInfo.blendEnabled)
				.srcColorBlendFactor(stateInfo.srcColorBlendFactor)
				.dstColorBlendFactor(stateInfo.dstColorBlendFactor)
				.colorBlendOp(stateInfo.blendOp)

				.srcAlphaBlendFactor(stateInfo.srcAlphaBlendFactor)
				.dstAlphaBlendFactor(stateInfo.dstAlphaBlendFactor)
				.alphaBlendOp(stateInfo.blendOp)

			val colourBlending: VkPipelineColorBlendStateCreateInfo =
				VkPipelineColorBlendStateCreateInfo.callocStack(it)
					.sType(VK10.VK_STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO)
					.logicOpEnable(stateInfo.colorLogicOpEnabled)
					.logicOp(stateInfo.colorLogicOp)
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
			VK10.vkCreatePipelineLayout(device.rawDevice, pipelineLayoutInfo, null, pPipelineLayout).ok()
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
			VK10.vkCreateGraphicsPipelines(device.rawDevice, VK10.VK_NULL_HANDLE, pipelineInfo, null, pGraphicsPipeline)
				.ok()
			graphicsPipeline = pGraphicsPipeline[0]

			VK10.vkDestroyShaderModule(device.rawDevice, vertShaderModule, null)
			VK10.vkDestroyShaderModule(device.rawDevice, fragShaderModule, null)

			return PipelineInfo(pipelineLayout, graphicsPipeline)
		}
	}
}
