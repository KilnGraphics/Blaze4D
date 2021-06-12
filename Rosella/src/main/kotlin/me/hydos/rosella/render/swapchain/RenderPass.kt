package me.hydos.rosella.render.swapchain

import me.hydos.rosella.Rosella
import me.hydos.rosella.render.device.Device
import me.hydos.rosella.render.util.ok
import org.lwjgl.system.MemoryStack.stackPush
import org.lwjgl.vulkan.*
import org.lwjgl.vulkan.KHRSwapchain.VK_IMAGE_LAYOUT_PRESENT_SRC_KHR
import org.lwjgl.vulkan.VK10.*
import java.nio.LongBuffer


class RenderPass(val device: Device, private val swapChain: SwapChain, private val engine: Rosella) {
	var renderPass: Long = 0

	init {
		stackPush().use {
			val attachments = VkAttachmentDescription.callocStack(2, it)
			val attachmentRefs = VkAttachmentReference.callocStack(2, it)

			attachments[0]
				.format(swapChain.swapChainImageFormat)
				.samples(VK_SAMPLE_COUNT_1_BIT)
				.loadOp(VK_ATTACHMENT_LOAD_OP_CLEAR)
				.storeOp(VK_ATTACHMENT_STORE_OP_STORE)
				.stencilLoadOp(VK_ATTACHMENT_LOAD_OP_DONT_CARE)
				.stencilStoreOp(VK_ATTACHMENT_STORE_OP_DONT_CARE)
				.initialLayout(VK_IMAGE_LAYOUT_UNDEFINED)
				.finalLayout(VK_IMAGE_LAYOUT_PRESENT_SRC_KHR)

			attachments[0].samples()

			val colorAttachmentRef = attachmentRefs[0]
				.attachment(0)
				.layout(VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL)

			attachments[1]
				.format(engine.renderer.depthBuffer.findDepthFormat(engine.device))
				.samples(VK_SAMPLE_COUNT_1_BIT)
				.loadOp(VK_ATTACHMENT_LOAD_OP_CLEAR)
				.storeOp(VK_ATTACHMENT_STORE_OP_DONT_CARE)
				.stencilLoadOp(VK_ATTACHMENT_LOAD_OP_DONT_CARE)
				.stencilStoreOp(VK_ATTACHMENT_STORE_OP_DONT_CARE)
				.initialLayout(VK_IMAGE_LAYOUT_UNDEFINED)
				.finalLayout(VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL)

			val depthAttachmentRef = attachmentRefs[1]
			depthAttachmentRef.attachment(1)
			depthAttachmentRef.layout(VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL)

			val subpass = VkSubpassDescription.callocStack(1, it)
				.pipelineBindPoint(VK_PIPELINE_BIND_POINT_GRAPHICS)
				.colorAttachmentCount(1)
				.pColorAttachments(VkAttachmentReference.callocStack(1, it).put(0, colorAttachmentRef))
				.pDepthStencilAttachment(depthAttachmentRef)

			val dependency = VkSubpassDependency.callocStack(1, it)
				.srcSubpass(VK_SUBPASS_EXTERNAL)
				.dstSubpass(0)
				.srcStageMask(VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT)
				.srcAccessMask(0)
				.dstStageMask(VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT)
				.dstAccessMask(VK_ACCESS_COLOR_ATTACHMENT_READ_BIT or VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT)

			val renderPassInfo: VkRenderPassCreateInfo = VkRenderPassCreateInfo.callocStack(it)
				.sType(VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO)
				.pAttachments(attachments)
				.pSubpasses(subpass)
				.pDependencies(dependency)

			val pRenderPass: LongBuffer = it.mallocLong(1)
			vkCreateRenderPass(device.device, renderPassInfo, null, pRenderPass).ok()
			renderPass = pRenderPass[0]
		}
	}
}

