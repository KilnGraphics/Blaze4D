package me.hydos.rosella.render.renderer;

import it.unimi.dsi.fastutil.ints.Int2ObjectOpenHashMap;
import it.unimi.dsi.fastutil.objects.ObjectArrayList;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.device.VulkanDevice;
import me.hydos.rosella.device.VulkanQueues;
import me.hydos.rosella.display.Display;
import me.hydos.rosella.memory.Memory;
import me.hydos.rosella.memory.buffer.GlobalBufferManager;
import me.hydos.rosella.render.VkKt;
import me.hydos.rosella.render.info.InstanceInfo;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.shader.RawShaderProgram;
import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.swapchain.DepthBuffer;
import me.hydos.rosella.render.swapchain.Frame;
import me.hydos.rosella.render.swapchain.RenderPass;
import me.hydos.rosella.render.swapchain.Swapchain;
import me.hydos.rosella.scene.object.impl.SimpleObjectManager;
import me.hydos.rosella.util.Color;
import me.hydos.rosella.vkobjects.VkCommon;
import org.lwjgl.PointerBuffer;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.vulkan.*;

import java.nio.IntBuffer;
import java.nio.LongBuffer;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;

import static me.hydos.rosella.render.util.VkUtilsKt.ok;
import static org.lwjgl.vulkan.VK10.*;

/**
 * Handles the bulk of Vulkan rendering
 */
public class Renderer {

    // Rosella instance this is owned to
    private final Rosella rosella;

    // For convenience instead of rosella.common
    private final VkCommon common;

    // The presentation and graphics queue
    public final VulkanQueues queues;

    // The depth buffer as Vulkan forces us to create our own
    public final DepthBuffer depthBuffer;

    // Should the swap chain be recreated next render
    private boolean recreateSwapChain;

    // The clear color
    private Color clearColor = new Color(50, 50, 50, 0);

    private List<Frame> inFlightFrames = new ObjectArrayList<>();
    private Map<Integer, Frame> imagesInFlight = new Int2ObjectOpenHashMap<>();
    private int currentFrameInFlight = 0;

    public Renderer(Rosella rosella) {
        this.rosella = rosella;
        this.common = rosella.common;

        this.queues = new VulkanQueues(common);
        this.depthBuffer = new DepthBuffer();

        VkKt.createCmdPool(common.device, this, common.surface);
        createSwapChain(common, common.display, ((SimpleObjectManager) rosella.objectManager));
    }

    public Swapchain swapchain;
    public RenderPass renderPass;

    public long commandPool = 0;
    List<VkCommandBuffer> commandBuffers = new ObjectArrayList<>();

    private void createSwapChain(VkCommon common, Display display, SimpleObjectManager objectManager) {
        this.swapchain = new Swapchain(display, common.device.rawDevice, common.device.physicalDevice, common.surface);
        this.renderPass = new RenderPass(common.device, swapchain, this);
        VkKt.createImgViews(swapchain, common.device);
        for (Material material : objectManager.materials) {
            material.pipeline = objectManager.pipelineManager.getPipeline(material, this);
        }
        depthBuffer.createDepthResources(common.device, swapchain, this);
        createFrameBuffers();
//      engine.camera.createViewAndProj(swapchain)
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

    public void render() {
        try (MemoryStack stack = MemoryStack.stackPush()) {
            Frame thisFrame = inFlightFrames.get(currentFrameInFlight);
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

            if (vkResult == KHRSwapchain.VK_ERROR_OUT_OF_DATE_KHR || vkResult == KHRSwapchain.VK_SUBOPTIMAL_KHR || recreateSwapChain) {
                recreateSwapChain = false;
                recreateSwapChain(rosella.common.display, rosella);
                ((SimpleObjectManager) rosella.objectManager).pipelineManager.invalidatePipelines(swapchain, rosella);
            } else if (vkResult != VK_SUCCESS) {
                throw new RuntimeException("Failed to present swap chain image");
            }

            ok(vkDeviceWaitIdle(rosella.common.device.rawDevice));

            currentFrameInFlight = (currentFrameInFlight + 1) % MAX_FRAMES_IN_FLIGHT;
        }
    }

    private void recreateSwapChain(Display window, Rosella rosella) {
        try (MemoryStack stack = MemoryStack.stackPush()) {
            IntBuffer width = stack.ints(0);
            IntBuffer height = stack.ints(0);
            while (width.get(0) == 0 && height.get(0) == 0) {
                window.waitForNonZeroSize();
            }
        }

        rosella.common.device.waitForIdle();
        freeSwapChain();
        createSwapChain(rosella.common, window, ((SimpleObjectManager) rosella.objectManager));
    }

    public void freeSwapChain() {
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
        inFlightFrames = new ObjectArrayList<>(MAX_FRAMES_IN_FLIGHT);
        imagesInFlight = new Int2ObjectOpenHashMap<>(swapchain.getSwapChainImages().size());

        try (MemoryStack stack = MemoryStack.stackPush()) {

            VkSemaphoreCreateInfo semaphoreInfo = VkSemaphoreCreateInfo.callocStack(stack);
            semaphoreInfo.sType(VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO);
            VkFenceCreateInfo fenceInfo = VkFenceCreateInfo.callocStack(stack);
            fenceInfo.sType(VK_STRUCTURE_TYPE_FENCE_CREATE_INFO);
            fenceInfo.flags(VK_FENCE_CREATE_SIGNALED_BIT);
            LongBuffer pImageAvailableSemaphore = stack.mallocLong(1);
            LongBuffer pRenderFinishedSemaphore = stack.mallocLong(1);
            LongBuffer pFence = stack.mallocLong(1);
            for (int i = 0; i < MAX_FRAMES_IN_FLIGHT; i++) {
                ok(vkCreateSemaphore(
                        common.device.rawDevice,
                        semaphoreInfo,
                        null,
                        pImageAvailableSemaphore
                ));
                ok(vkCreateSemaphore(
                        common.device.rawDevice,
                        semaphoreInfo,
                        null,
                        pRenderFinishedSemaphore
                ));
                ok(vkCreateFence(common.device.rawDevice, fenceInfo, null, pFence));
                inFlightFrames.add(
                        new Frame(
                                pImageAvailableSemaphore.get(0),
                                pRenderFinishedSemaphore.get(0),
                                pFence.get(0)
                        )
                );
            }
        }
    }

    public void windowResizeCallback(int width, int height) {
        this.recreateSwapChain = true;
    }

    private void createFrameBuffers() {
        swapchain.setFrameBuffers(new ArrayList<>(swapchain.getSwapChainImageViews().size()));

        try (MemoryStack stack = MemoryStack.stackPush()) {
            LongBuffer attachments = stack.longs(VK_NULL_HANDLE, depthBuffer.getDepthImageView());
            LongBuffer pFramebuffer = stack.mallocLong(1);
            VkFramebufferCreateInfo framebufferInfo = VkFramebufferCreateInfo.callocStack(stack)
                    .sType(VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO)
                    .renderPass(renderPass.getRenderPass())
                    .width(swapchain.getSwapChainExtent().width())
                    .height(swapchain.getSwapChainExtent().height())
                    .layers(1);
            for (long imageView : swapchain.getSwapChainImageViews()) {
                attachments.put(0, imageView);
                framebufferInfo.pAttachments(attachments);
                ok(vkCreateFramebuffer(common.device.rawDevice, framebufferInfo, null, pFramebuffer));
                swapchain.getFrameBuffers().add(pFramebuffer.get(0));
            }
        }
    }

    /**
     * Create the Command Buffers
     */
    public void rebuildCommandBuffers(RenderPass renderPass, SimpleObjectManager simpleObjectManager) {
        simpleObjectManager.rebuildCmdBuffers(renderPass, null, null); //TODO: move it into here
        List<ShaderProgram> usedShaders = new ArrayList<>();
        for (Material material : simpleObjectManager.materials) {
            if (!usedShaders.contains(material.getShader())) {
                usedShaders.add(material.getShader());
            }
        }

        for (List<InstanceInfo> instances : simpleObjectManager.renderObjects.values()) {
            for (InstanceInfo instance : instances) {
                instance.rebuild(rosella);
            }
        }

        try (MemoryStack stack = MemoryStack.stackPush()) {

            int commandBuffersCount = swapchain.getFrameBuffers().size();

            commandBuffers = new ObjectArrayList<>(commandBuffersCount);

            PointerBuffer pCommandBuffers = VkKt.allocateCmdBuffers(
                    stack,
                    common.device,
                    commandPool,
                    commandBuffersCount,
                    VK_COMMAND_BUFFER_LEVEL_PRIMARY
            );

            for (int i = 0; i < commandBuffersCount; i++) {
                commandBuffers.add(
                        new VkCommandBuffer(
                                pCommandBuffers.get(i),
                                common.device.rawDevice
                        )
                );
            }

            VkCommandBufferBeginInfo beginInfo = VkKt.createBeginInfo(stack);
            VkRenderPassBeginInfo renderPassInfo = VkKt.createRenderPassInfo(stack, renderPass);
            VkRect2D renderArea = VkKt.createRenderArea(stack, 0, 0, swapchain);
            VkClearValue.Buffer clearValues = VkKt.createClearValues(stack, clearColor.rAsFloat(), clearColor.gAsFloat(), clearColor.bAsFloat(), 1.0f, 0);

            renderPassInfo.renderArea(renderArea)
                    .pClearValues(clearValues);

            for (int i = 0; i < commandBuffersCount; i++) {
                VkCommandBuffer commandBuffer = commandBuffers.get(i);
                ok(vkBeginCommandBuffer(commandBuffer, beginInfo));
                renderPassInfo.framebuffer(swapchain.getFrameBuffers().get(i));

                vkCmdBeginRenderPass(commandBuffer, renderPassInfo, VK_SUBPASS_CONTENTS_INLINE);

                if (rosella.bufferManager != null && !simpleObjectManager.renderObjects.isEmpty()) {
                    int finalI = i;
                    bindBigBuffers(rosella.bufferManager, stack, commandBuffer);
                    simpleObjectManager.renderObjects.keySet().forEach(renderInfo -> {
                                bindBigBuffers(rosella.bufferManager, stack, commandBuffer);
                                for (InstanceInfo instance : simpleObjectManager.renderObjects.get(renderInfo)) {
                                    bindInstanceInfo(instance, stack, commandBuffer, finalI);
                                    vkCmdDrawIndexed(commandBuffer, renderInfo.getIndicesSize(), 1, rosella.bufferManager.indicesOffsetMap.get(renderInfo), rosella.bufferManager.vertexOffsetMap.get(renderInfo), 0);
                                }
                            }
                    );

                    vkCmdEndRenderPass(commandBuffer);
                    ok(vkEndCommandBuffer(commandBuffer));
                }
            }
        }
    }

    private void bindBigBuffers(
            GlobalBufferManager bufferManager,
            MemoryStack stack,
            VkCommandBuffer commandBuffer
    ) {
        LongBuffer offsets = stack.longs(0);
        LongBuffer vertexBuffers = stack.longs(bufferManager.vertexBuffer.buffer());
        vkCmdBindVertexBuffers(commandBuffer, 0, vertexBuffers, offsets);
        vkCmdBindIndexBuffer(commandBuffer, bufferManager.indexBuffer.buffer(), 0, VK_INDEX_TYPE_UINT32);
    }

    private void bindInstanceInfo(
            InstanceInfo instanceInfo,
            MemoryStack matrix,
            VkCommandBuffer commandBuffer,
            int commandBufferIndex
    ) {
        vkCmdBindPipeline(
                commandBuffer,
                VK_PIPELINE_BIND_POINT_GRAPHICS,
                instanceInfo.material().pipeline.getGraphicsPipeline()
        );

        vkCmdBindDescriptorSets(
                commandBuffer,
                VK_PIPELINE_BIND_POINT_GRAPHICS,
                instanceInfo.material().pipeline.getPipelineLayout(),
                0,
                matrix.longs(instanceInfo.ubo().getDescriptors().getDescriptorSets().get(commandBufferIndex)),
                null
        );
    }

    public void clearColor(Color color) {
        if (clearColor != color) {
            clearColor = color;
            rebuildCommandBuffers(renderPass, ((SimpleObjectManager) rosella.objectManager));
        }
    }

    public void teardown() {
        freeSwapChain();

        for (Frame frame : inFlightFrames) {
            vkDestroySemaphore(common.device.rawDevice, frame.renderFinishedSemaphore(), null);
            vkDestroySemaphore(common.device.rawDevice, frame.imageAvailableSemaphore(), null);
            vkDestroyFence(common.device.rawDevice, frame.fence(), null);
        }
    }

    public static final int MAX_FRAMES_IN_FLIGHT = 2;
    public static final long UINT64_MAX = -0x1L;
}
