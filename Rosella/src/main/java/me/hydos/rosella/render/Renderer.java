package me.hydos.rosella.render;

import java.nio.IntBuffer;
import java.util.List;
import java.util.Map;

import it.unimi.dsi.fastutil.ints.Int2ObjectOpenHashMap;
import it.unimi.dsi.fastutil.objects.ObjectArrayList;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.device.VulkanDevice;
import me.hydos.rosella.device.VulkanQueues;
import me.hydos.rosella.display.Display;
import me.hydos.rosella.memory.Memory;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.shader.RawShaderProgram;
import me.hydos.rosella.render.shader.Shader;
import me.hydos.rosella.render.swapchain.DepthBuffer;
import me.hydos.rosella.render.swapchain.Frame;
import me.hydos.rosella.render.swapchain.RenderPass;
import me.hydos.rosella.render.swapchain.Swapchain;
import me.hydos.rosella.scene.object.impl.SimpleObjectManager;
import me.hydos.rosella.vkobjects.VkCommon;
import org.lwjgl.PointerBuffer;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.vulkan.*;

import static me.hydos.rosella.render.util.VkUtilsKt.ok;
import static org.lwjgl.vulkan.VK10.*;

public class Renderer {
    private final VkCommon common;
    private final Display display;
    private final Rosella rosella;

    private DepthBuffer depthBuffer = new DepthBuffer();

    public Renderer(VkCommon common, Display display, Rosella rosella) {
        this.common = common;
        this.display = display;
        this.rosella = rosella;

        queues = new VulkanQueues(common);

        createCmdPool(common.device, this, common.surface);
        createSwapChain(common, display, ((SimpleObjectManager) rosella.objectManager));
    }

        private List<Frame> inFlightFrames = new ObjectArrayList<>();
        private Map<Integer, Frame> imagesInFlight = new Int2ObjectOpenHashMap<>();
        private int currentFrame = 0;

        private boolean resizeFramebuffer = false;

        private float r = 0.2f;
        private float g = 0.2f;
        private float b = 0.2f;

        Swapchain swapchain;
        RenderPass renderPass;

        VulkanQueues queues;

        long commandPool = 0;
        List<VkCommandBuffer> commandBuffers = new ObjectArrayList<VkCommandBuffer>();


        private void createSwapChain(VkCommon common, Display display, SimpleObjectManager objectManager) {
            this.swapchain = new Swapchain(display, common.device.rawDevice, common.device.physicalDevice, common.surface);
            this.renderPass = new RenderPass(common.device, swapchain, this);
            createImgViews(swapchain, common.device);
            for (Material material : objectManager.materials) {
                material.pipeline = objectManager.pipelineManager.getPipeline(material, this);
            }
            depthBuffer.createDepthResources(common.device, swapchain, this);
            createFrameBuffers();
//		engine.camera.createViewAndProj(swapchain)
            rebuildCommandBuffers(renderPass, objectManager);
            createSyncObjects();
        }

        public VkCommandBuffer beginCmdBuffer(MemoryStack stack, PointerBuffer pCommandBuffer, VulkanDevice device) {
            VkCommandBufferAllocateInfo allocInfo = VkCommandBufferAllocateInfo.callocStack(stack)
                    .sType(VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO)
                    .level(VK_COMMAND_BUFFER_LEVEL_PRIMARY)
                    .commandPool(commandPool)
                    .commandBufferCount(1);
            ok(vkAllocateCommandBuffers(device.rawDevice, allocInfo, pCommandBuffer));
            VkCommandBuffer commandBuffer = new VkCommandBuffer(pCommandBuffer.get(0), device.rawDevice);
            VkCommandBufferBeginInfo beginInfo = VkCommandBufferBeginInfo.callocStack(stack)
                    .sType(VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO)
                    .flags(VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT);
            ok(vkBeginCommandBuffer(commandBuffer, beginInfo));
            return commandBuffer;
        }

        public void render(Rosella rosella) {
            try(MemoryStack stack = MemoryStack.stackPush()) {
                Frame thisFrame = inFlightFrames.get(currentFrame);
                ok(vkWaitForFences(rosella.common.device.rawDevice, thisFrame.pFence(), true, UINT64_MAX));

                IntBuffer pImageIndex = stack.mallocInt(1);

                int vkResult = KHRSwapchain.vkAcquireNextImageKHR(
                        rosella.common.device.rawDevice,
                        swapchain.getSwapChain(),
                        UINT64_MAX,
                        thisFrame.imageAvailableSemaphore(),
                        VK_NULL_HANDLE,
                        pImageIndex
                );

                if (vkResult == KHRSwapchain.VK_ERROR_OUT_OF_DATE_KHR) {
                    recreateSwapChain(rosella.common.display, rosella);
                    return;
                }

                int imageIndex = pImageIndex.get(0);

                for (RawShaderProgram shader : (((SimpleObjectManager) rosella.objectManager)).shaderManager.getCachedShaders().keySet()) {
                    shader.prepareTexturesForRender(rosella.renderer, ((SimpleObjectManager) rosella.objectManager).textureManager);
                    shader.updateUbos(imageIndex, swapchain, (SimpleObjectManager) rosella.objectManager);
                }

                if (imagesInFlight.containsKey(imageIndex)) {
                    ok(vkWaitForFences(
                            rosella.common.device.rawDevice,
                            imagesInFlight.get(imageIndex).fence(),
                            true,
                            UINT64_MAX
				    ));
                }

                imagesInFlight.put(imageIndex, thisFrame);
                VkSubmitInfo submitInfo = VkSubmitInfo.callocStack(stack)
                        .sType(VK_STRUCTURE_TYPE_SUBMIT_INFO)
                        .waitSemaphoreCount(1)
                        .pWaitSemaphores(thisFrame.pImageAvailableSemaphore())
                        .pWaitDstStageMask(stack.ints(VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT))
                        .pSignalSemaphores(thisFrame.pRenderFinishedSemaphore())
                        .pCommandBuffers(stack.pointers(commandBuffers.get(imageIndex)));

                ok(vkResetFences(rosella.common.device.rawDevice, thisFrame.pFence()));
                ok(vkQueueSubmit(queues.graphicsQueue, submitInfo, thisFrame.fence()));

                VkPresentInfoKHR presentInfo = VkPresentInfoKHR.callocStack(stack)
                        .sType(KHRSwapchain.VK_STRUCTURE_TYPE_PRESENT_INFO_KHR)
                        .pWaitSemaphores(thisFrame.pRenderFinishedSemaphore())
                        .swapchainCount(1)
                        .pSwapchains(stack.longs(swapchain.getSwapChain()))
                        .pImageIndices(pImageIndex);

                vkResult = KHRSwapchain.vkQueuePresentKHR(queues.presentQueue, presentInfo);

                if (vkResult == KHRSwapchain.VK_ERROR_OUT_OF_DATE_KHR || vkResult == KHRSwapchain.VK_SUBOPTIMAL_KHR || resizeFramebuffer) {
                    resizeFramebuffer = false;
                    recreateSwapChain(rosella.common.display, rosella);
                    ((SimpleObjectManager) rosella.objectManager).pipelineManager.invalidatePipelines(swapchain, rosella);
                } else if (vkResult != VK_SUCCESS) {
                    throw new RuntimeException("Failed to present swap chain image");
                }

                ok(vkDeviceWaitIdle(common.device.rawDevice));

                currentFrame = (currentFrame + 1) % MAX_FRAMES_IN_FLIGHT;
            }
        }

        private void recreateSwapChain(Display window, Rosella rosella) {
            try(MemoryStack stack = MemoryStack.stackPush()) {
                IntBuffer width = stack.ints(0);
                IntBuffer height = stack.ints(0);
                while (width.get(0) == 0 && height.get(0) == 0) {
                    window.waitForNonZeroSize();
                }
            }

            rosella.common.device.waitForIdle();
            freeSwapChain(rosella);
            createSwapChain(rosella.common, window, ((SimpleObjectManager) rosella.objectManager));
        }

        public void freeSwapChain(Rosella rosella) {
            for (RawShaderProgram shader : ((SimpleObjectManager) rosella.objectManager).shaderManager.getCachedShaders().keySet()) {
                vkDestroyDescriptorPool(rosella.common.device.rawDevice, shader.getDescriptorPool(), null);
            }

            clearCommandBuffers(rosella.common.device);

            // Free Depth Buffer
            depthBuffer.free(rosella.common.device);

            swapchain.getFrameBuffers().forEach(framebuffer ->
                    vkDestroyFramebuffer(
                            rosella.common.device.rawDevice,
                            framebuffer,
                            null
                    )
            );

            vkDestroyRenderPass(rosella.common.device.rawDevice, renderPass.getRenderPass(), null);
            swapchain.getSwapChainImageViews().forEach(imageView ->
                    vkDestroyImageView(
                            rosella.common.device.rawDevice,
                            imageView,
                            null
                    )
            );

            swapchain.free(rosella.common.device.rawDevice);
        }

        public void clearCommandBuffers(VulkanDevice device) {
            if (commandBuffers.size() != 0) {
                vkFreeCommandBuffers(device.rawDevice, commandPool, Memory.asPointerBuffer(commandBuffers));
                commandBuffers.clear();
            }
        }

        private void createSyncObjects() {
            inFlightFrames = new ObjectArrayList<Frame>(MAX_FRAMES_IN_FLIGHT);
            imagesInFlight = new Int2ObjectOpenHashMap<>()(swapchain.getSwapChainImages().size());

            MemoryStack.stackPush().use { stack ->
                    val semaphoreInfo = VkSemaphoreCreateInfo.callocStack(stack)
                semaphoreInfo.sType(VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO)
                val fenceInfo = VkFenceCreateInfo.callocStack(stack)
                fenceInfo.sType(VK_STRUCTURE_TYPE_FENCE_CREATE_INFO)
                fenceInfo.flags(VK_FENCE_CREATE_SIGNALED_BIT)
                val pImageAvailableSemaphore = stack.mallocLong(1)
                val pRenderFinishedSemaphore = stack.mallocLong(1)
                val pFence = stack.mallocLong(1)
                for (i in 0 until MAX_FRAMES_IN_FLIGHT) {
                    vkCreateSemaphore(
                            common.device.rawDevice,
                            semaphoreInfo,
                            null,
                            pImageAvailableSemaphore
                    ).ok()
                    vkCreateSemaphore(
                            common.device.rawDevice,
                            semaphoreInfo,
                            null,
                            pRenderFinishedSemaphore
                    ).ok()
                    vkCreateFence(common.device.rawDevice, fenceInfo, null, pFence).ok()
                    inFlightFrames.add(
                            Frame(
                                    pImageAvailableSemaphore[0],
                                    pRenderFinishedSemaphore[0],
                                    pFence[0]
                            )
                    )
                }
            }
        }

        fun windowResizeCallback(width: Int, height: Int) {
            this.resizeFramebuffer = true
        }

        private fun createFrameBuffers() {
            swapchain.frameBuffers = ArrayList(swapchain.swapChainImageViews.size)
            MemoryStack.stackPush().use { stack ->
                    val attachments = stack.longs(VK_NULL_HANDLE, depthBuffer.depthImageView)
                val pFramebuffer = stack.mallocLong(1)
                val framebufferInfo = VkFramebufferCreateInfo.callocStack(stack)
                        .sType(VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO)
                        .renderPass(renderPass.renderPass)
                        .width(swapchain.swapChainExtent.width())
                        .height(swapchain.swapChainExtent.height())
                        .layers(1)
                for (imageView in swapchain.swapChainImageViews) {
                    attachments.put(0, imageView)
                    framebufferInfo.pAttachments(attachments)
                    vkCreateFramebuffer(common.device.rawDevice, framebufferInfo, null, pFramebuffer).ok()
                    swapchain.frameBuffers.add(pFramebuffer[0])
                }
            }
        }

        /**
         * Create the Command Buffers
         */
        fun rebuildCommandBuffers(renderPass: RenderPass, simpleObjectManager: SimpleObjectManager) {
            simpleObjectManager.rebuildCmdBuffers(renderPass, null, null) //TODO: move it into here
            val usedShaders = ArrayList<ShaderProgram>()
            for (material in simpleObjectManager.materials) {
                if (!usedShaders.contains(material.shader)) {
                    usedShaders.add(material.shader!!)
                }
            }

            for (instances in simpleObjectManager.renderObjects.values) {
                for (instance in instances) {
                    instance.rebuild(rosella)
                }
            }

            MemoryStack.stackPush().use {
                val commandBuffersCount: Int = swapchain.frameBuffers.size

                commandBuffers = ArrayList(commandBuffersCount)

                val pCommandBuffers = allocateCmdBuffers(
                        it,
                        common.device,
                        commandPool,
                        commandBuffersCount
                )

                for (i in 0 until commandBuffersCount) {
                    commandBuffers.add(
                            VkCommandBuffer(
                                    pCommandBuffers[i],
                                    common.device.rawDevice
                            )
                    )
                }

                val beginInfo = createBeginInfo(it)
                val renderPassInfo = createRenderPassInfo(it, renderPass)
                val renderArea = createRenderArea(it, 0, 0, swapchain)
                val clearValues = createClearValues(it, r, g, b, 1.0f, 0)

                renderPassInfo.renderArea(renderArea)
                        .pClearValues(clearValues)

                for (i in 0 until commandBuffersCount) {
                    val commandBuffer = commandBuffers[i]
                    vkBeginCommandBuffer(commandBuffer, beginInfo).ok()
                    renderPassInfo.framebuffer(swapchain.frameBuffers[i])

                    vkCmdBeginRenderPass(commandBuffer, renderPassInfo, VK_SUBPASS_CONTENTS_INLINE)
                    if (rosella.bufferManager != null && simpleObjectManager.renderObjects.isNotEmpty()) {
                        simpleObjectManager.renderObjects.keys.forEach { renderInfo ->
                                bindBigBuffers(rosella.bufferManager, setOf(renderInfo), it, commandBuffer)
                            for (instance in simpleObjectManager.renderObjects[renderInfo]!!) {
                                bindInstanceInfo(instance, it, commandBuffer, i)
                                vkCmdDrawIndexed(commandBuffer, renderInfo.indicesSize, 1, 0, 0, 0)
                            }
                        }
                    }
                    vkCmdEndRenderPass(commandBuffer)

                    vkEndCommandBuffer(commandBuffer).ok()
                }
            }
        }

        private fun bindBigBuffers(
                bufferManager: GlobalBufferManager,
                renderInfos: Set<RenderInfo>,
        stack: MemoryStack,
                commandBuffer: VkCommandBuffer
	) {
            val vertexBuffer = bufferManager.createVertexBuffer(renderInfos)
            val indexBuffer = bufferManager.createIndexBuffer(renderInfos)

            val offsets = stack.longs(0)
            val vertexBuffers = stack.longs(vertexBuffer.buffer)
            vkCmdBindVertexBuffers(commandBuffer, 0, vertexBuffers, offsets)
            vkCmdBindIndexBuffer(commandBuffer, indexBuffer.buffer, 0, VK_INDEX_TYPE_UINT32)
        }

        private fun bindInstanceInfo(
                instanceInfo: InstanceInfo,
                matrix: MemoryStack,
                commandBuffer: VkCommandBuffer,
                commandBufferIndex: Int
	) {
            vkCmdBindPipeline(
                    commandBuffer,
                    VK_PIPELINE_BIND_POINT_GRAPHICS,
                    instanceInfo.material.pipeline.graphicsPipeline
            )

            vkCmdBindDescriptorSets(
                    commandBuffer,
                    VK_PIPELINE_BIND_POINT_GRAPHICS,
                    instanceInfo.material.pipeline.pipelineLayout,
                    0,
                    matrix.longs(instanceInfo.ubo.getDescriptors().descriptorSets[commandBufferIndex]),
                    null
            )
        }

        fun clearColor(red: Float, green: Float, blue: Float, rosella: Rosella) {
            if (this.r != red || this.g != green || this.b != blue) {
                this.r = red
                this.g = green
                this.b = blue
                rebuildCommandBuffers(renderPass, rosella.objectManager as SimpleObjectManager)
            }
        }

        companion object {
		const val MAX_FRAMES_IN_FLIGHT = 2
		const val UINT64_MAX = -0x1L
        }
}
