package me.hydos.rosella.render.material

import me.hydos.rosella.Rosella
import me.hydos.rosella.render.device.Device
import me.hydos.rosella.render.model.Vertex
import me.hydos.rosella.render.resource.Identifier
import me.hydos.rosella.render.resource.Resource
import me.hydos.rosella.render.shader.ShaderProgram
import me.hydos.rosella.render.swapchain.RenderPass
import me.hydos.rosella.render.swapchain.SwapChain
import me.hydos.rosella.render.texture.Texture
import me.hydos.rosella.render.util.*
import org.joml.Vector3f
import org.lwjgl.system.MemoryStack
import org.lwjgl.vulkan.*
import org.lwjgl.vulkan.VK10.*
import java.nio.ByteBuffer
import java.nio.LongBuffer

/**
 * A Material is like texture information, normal information, and all of those things which give an object character wrapped into one class.
 * similar to how unity material's works
 * guaranteed to change once and a while
 */
class Material(
	val resource: Resource,
	private val shaderId: Identifier,
	private val imgFormat: Int,
	private val useBlend: Boolean
) {
	var pipelineLayout: Long = 0
	var graphicsPipeline: Long = 0

	lateinit var shader: ShaderProgram

	lateinit var texture: Texture

	fun loadShaders(engine: Rosella) {
		val retrievedShader = engine.shaderManager.getOrCreateShader(shaderId)
			?: error("The shader $shaderId couldn't be found. (Are you registering it?)")
		this.shader = retrievedShader
	}

	fun loadTextures(engine: Rosella) {
		if (resource != Resource.Empty) {
			texture = engine.textureManager.getOrLoadTexture(resource, engine, imgFormat)!!
		}
	}

	/**
	 * The main rendering pipeline of this material
	 */
	fun createPipeline(
		device: Device,
		swapChain: SwapChain,
		renderPass: RenderPass,
		descriptorSetLayout: Long,
		polygonMode: Int
	) {
		MemoryStack.stackPush().use {
			val vertShaderModule = shader.getVertShaderModule()
			val fragShaderModule = shader.getFragShaderModule()

			val entryPoint: ByteBuffer = it.UTF8("main")
			val shaderStages = VkPipelineShaderStageCreateInfo.callocStack(2, it)

			shaderStages[0]
				.sType(VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO)
				.stage(VK_SHADER_STAGE_VERTEX_BIT)
				.module(vertShaderModule)
				.pName(entryPoint)

			shaderStages[1]
				.sType(VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO)
				.stage(VK_SHADER_STAGE_FRAGMENT_BIT)
				.module(fragShaderModule)
				.pName(entryPoint)

			/**
			 * Vertex
			 */
			val vertexInputInfo: VkPipelineVertexInputStateCreateInfo =
				VkPipelineVertexInputStateCreateInfo.callocStack(it)
					.sType(VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO)
					.pVertexBindingDescriptions(Vertex.bindingDescription)
					.pVertexAttributeDescriptions(Vertex.attributeDescriptions)

			/**
			 * Assembly
			 */
			val inputAssembly: VkPipelineInputAssemblyStateCreateInfo =
				VkPipelineInputAssemblyStateCreateInfo.callocStack(it)
					.sType(VK_STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO)
					.topology(VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST)
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
				.sType(VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO)
				.pViewports(viewport)
				.pScissors(scissor)

			/**
			 * Rasterisation
			 */
			val rasterizer: VkPipelineRasterizationStateCreateInfo =
				VkPipelineRasterizationStateCreateInfo.callocStack(it)
					.sType(VK_STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO)
					.depthClampEnable(false)
					.rasterizerDiscardEnable(false)
					.polygonMode(polygonMode)
					.lineWidth(1.0f)
					.cullMode(VK_CULL_MODE_BACK_BIT)
					.frontFace(VK_FRONT_FACE_COUNTER_CLOCKWISE)
					.depthBiasEnable(false)

			/**
			 * Multisampling
			 */
			val multisampling: VkPipelineMultisampleStateCreateInfo =
				VkPipelineMultisampleStateCreateInfo.callocStack(it)
					.sType(VK_STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO)
					.sampleShadingEnable(false)
					.rasterizationSamples(VK_SAMPLE_COUNT_1_BIT)

			val depthStencil: VkPipelineDepthStencilStateCreateInfo =
				VkPipelineDepthStencilStateCreateInfo.callocStack(it)
					.sType(VK_STRUCTURE_TYPE_PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO)
					.depthTestEnable(true)
					.depthWriteEnable(true)
					.depthCompareOp(VK_COMPARE_OP_LESS)
					.depthBoundsTestEnable(false)
					.minDepthBounds(0.0f)
					.maxDepthBounds(1.0f)
					.stencilTestEnable(false)

			/**
			 * Colour Blending
			 */
			val colourBlendAttachment = VkPipelineColorBlendAttachmentState.callocStack(1, it)
				.colorWriteMask(VK_COLOR_COMPONENT_R_BIT or VK_COLOR_COMPONENT_G_BIT or VK_COLOR_COMPONENT_B_BIT or VK_COLOR_COMPONENT_A_BIT)
				.blendEnable(useBlend)
				.srcColorBlendFactor(VK_BLEND_FACTOR_SRC_ALPHA)
				.dstColorBlendFactor(VK_BLEND_FACTOR_ONE_MINUS_SRC_ALPHA)
				.colorBlendOp(VK_BLEND_OP_ADD)

				.srcAlphaBlendFactor(VK_BLEND_FACTOR_ZERO)
				.dstAlphaBlendFactor(VK_BLEND_FACTOR_ONE_MINUS_SRC_ALPHA)
				.alphaBlendOp(VK_BLEND_OP_SUBTRACT)

			val colourBlending: VkPipelineColorBlendStateCreateInfo =
				VkPipelineColorBlendStateCreateInfo.callocStack(it)
					.sType(VK_STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO)
					.logicOpEnable(false)
					.logicOp(VK_LOGIC_OP_COPY)
					.pAttachments(colourBlendAttachment)
					.blendConstants(it.floats(0.0f, 0.0f, 0.0f, 0.0f))

			/**
			 * Create Push Constants
			 */
			val pushConstantRange = VkPushConstantRange.callocStack(1, it)
				.stageFlags(VK_SHADER_STAGE_VERTEX_BIT)
				.offset(0)
				.size(sizeof(Vector3f::class))

			/**
			 * Pipeline Layout Creation
			 */
			val pipelineLayoutInfo: VkPipelineLayoutCreateInfo = VkPipelineLayoutCreateInfo.callocStack(it)
				.sType(VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO)
				.pSetLayouts(it.longs(descriptorSetLayout))
				.pPushConstantRanges(pushConstantRange)

			val pPipelineLayout = it.longs(VK_NULL_HANDLE)
			vkCreatePipelineLayout(device.device, pipelineLayoutInfo, null, pPipelineLayout).ok()
			pipelineLayout = pPipelineLayout[0]

			/**
			 * Pipeline Creation
			 */
			val pipelineInfo = VkGraphicsPipelineCreateInfo.callocStack(1, it)
				.sType(VK_STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO)
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
				.basePipelineHandle(VK_NULL_HANDLE)
				.basePipelineIndex(-1)

			val pGraphicsPipeline: LongBuffer = it.mallocLong(1)
			vkCreateGraphicsPipelines(device.device, VK_NULL_HANDLE, pipelineInfo, null, pGraphicsPipeline)
				.ok()
			graphicsPipeline = pGraphicsPipeline[0]

			vkDestroyShaderModule(device.device, vertShaderModule, null)
			vkDestroyShaderModule(device.device, fragShaderModule, null)
		}
	}

	fun free(device: Device) {
		vkDestroyPipeline(device.device, graphicsPipeline, null)
		vkDestroyPipelineLayout(device.device, pipelineLayout, null)
	}
}
