package me.hydos.rosella.render.renderer;

import it.unimi.dsi.fastutil.ints.Int2ObjectOpenHashMap;
import it.unimi.dsi.fastutil.objects.ObjectArrayList;
import me.hydos.rosella.Rosella;
import me.hydos.rosella.device.VulkanDevice;
import me.hydos.rosella.device.VulkanQueues;
import me.hydos.rosella.display.Display;
import me.hydos.rosella.memory.BufferInfo;
import me.hydos.rosella.memory.Memory;
import me.hydos.rosella.memory.buffer.GlobalBufferManager;
import me.hydos.rosella.render.VkKt;
import me.hydos.rosella.render.info.InstanceInfo;
import me.hydos.rosella.render.info.RenderInfo;
import me.hydos.rosella.render.material.Material;
import me.hydos.rosella.render.shader.RawShaderProgram;
import me.hydos.rosella.render.swapchain.DepthBuffer;
import me.hydos.rosella.render.swapchain.Frame;
import me.hydos.rosella.render.swapchain.RenderPass;
import me.hydos.rosella.render.swapchain.Swapchain;
import me.hydos.rosella.scene.object.impl.SimpleObjectManager;
import me.hydos.rosella.util.Color;
import me.hydos.rosella.vkobjects.VkCommon;
import org.lwjgl.PointerBuffer;
import org.lwjgl.system.MemoryStack;
import org.lwjgl.system.MemoryUtil;
import org.lwjgl.util.vma.VmaAllocationCreateInfo;
import org.lwjgl.vulkan.*;

import java.nio.IntBuffer;
import java.nio.LongBuffer;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.concurrent.TimeUnit;

import static me.hydos.rosella.render.util.VkUtilsKt.ok;
import static org.lwjgl.vulkan.KHRSwapchain.VK_IMAGE_LAYOUT_PRESENT_SRC_KHR;
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
    private boolean initialSwapchainCreated;
    private boolean recreateSwapChain;
    private boolean requireHardRebuild;

    // The clear color
    private Color clearColor = new Color(50, 50, 50, 0); // TODO: move this somewhere else, maybe in StateInfo?
    private float clearDepth = 1.0f;
    private int clearStencil = 0;

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
        initialSwapchainCreated = true;
    }

    public Swapchain swapchain;
    public RenderPass renderPass;

    public long commandPool = 0;
    List<VkCommandBuffer> commandBuffers = new ObjectArrayList<>();

    private void createSwapChain(VkCommon common, Display display, SimpleObjectManager objectManager) {
        this.swapchain = new Swapchain(display, common.device.rawDevice, common.device.physicalDevice, common.surface);
        this.renderPass = new RenderPass(common.device, swapchain, this);
        VkKt.createImgViews(swapchain, common.device);
        depthBuffer.createDepthResources(common.device, swapchain, this);
        createFrameBuffers();

        // Engine may still be initialising so we do a null check just in case
        if (objectManager.pipelineManager != null) {
            objectManager.pipelineManager.invalidatePipelines(common);
        }

        for (Material material : objectManager.materials) {
            material.pipeline = objectManager.pipelineManager.getPipeline(material, this);
        }

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

            if (vkResult == KHRSwapchain.VK_ERROR_OUT_OF_DATE_KHR || recreateSwapChain) {
                recreateSwapChain = false;
                requireHardRebuild = true;
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
            ok(queues.graphicsQueue.vkQueueSubmit(submitInfo, thisFrame.fence()));

            VkPresentInfoKHR presentInfo = VkPresentInfoKHR.callocStack(stack)
                    .sType(KHRSwapchain.VK_STRUCTURE_TYPE_PRESENT_INFO_KHR)
                    .pWaitSemaphores(thisFrame.pRenderFinishedSemaphore())
                    .swapchainCount(1)
                    .pSwapchains(stack.longs(swapchain.getSwapChain()))
                    .pImageIndices(pImageIndex);

            vkResult = queues.presentQueue.vkQueuePresentKHR(presentInfo);

            if (vkResult == KHRSwapchain.VK_ERROR_OUT_OF_DATE_KHR || vkResult == KHRSwapchain.VK_SUBOPTIMAL_KHR || recreateSwapChain) {
                // TODO OPT: add a lazier method for iconification, use the glfw callback for it
                recreateSwapChain = false;
                requireHardRebuild = true;
                recreateSwapChain(rosella.common.display, rosella);
            } else if (vkResult != VK_SUCCESS) {
                throw new RuntimeException("Failed to present swap chain image");
            }

            ok(vkDeviceWaitIdle(rosella.common.device.rawDevice));

            currentFrameInFlight = (currentFrameInFlight + 1) % MAX_FRAMES_IN_FLIGHT;
        }
    }

    private void recreateSwapChain(Display window, Rosella rosella) {
        while (window.width == 0 || window.height == 0) {
            window.waitForNonZeroSize();
        }

        rosella.common.device.waitForIdle();
        freeSwapChain();
        createSwapChain(rosella.common, window, ((SimpleObjectManager) rosella.objectManager));
    }

    public void freeSwapChain() {
        for (RawShaderProgram shader : ((SimpleObjectManager) rosella.objectManager).shaderManager.getCachedShaders().keySet()) {
            vkDestroyDescriptorPool(rosella.common.device.rawDevice, shader.getDescriptorPool(), null);
            shader.setDescriptorPool(0);
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

    public void queueRecreateSwapchain() {
        if (initialSwapchainCreated) {
            recreateSwapChain = true;
        }
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
        if (!recreateSwapChain) {
            simpleObjectManager.rebuildCmdBuffers(renderPass, null, null); //TODO: move it into here

            for (List<InstanceInfo> instances : simpleObjectManager.renderObjects.values()) {
                for (InstanceInfo instance : instances) {
                    if(requireHardRebuild) {
                        instance.hardRebuild(rosella);
                    } else {
                        instance.rebuild(rosella);
                    }
                }
            }
            requireHardRebuild = false;

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
                VkClearValue.Buffer clearValues = VkKt.createClearValues(stack, clearColor.rAsFloat(), clearColor.gAsFloat(), clearColor.bAsFloat(), clearDepth, clearStencil);

                renderPassInfo.renderArea(renderArea)
                        .pClearValues(clearValues);

                if (rosella.bufferManager != null && !simpleObjectManager.renderObjects.isEmpty()) {
                    rosella.bufferManager.nextFrame(simpleObjectManager.renderObjects.keySet());
                }

                for (int i = 0; i < commandBuffersCount; i++) {
                    VkCommandBuffer commandBuffer = commandBuffers.get(i);
                    ok(vkBeginCommandBuffer(commandBuffer, beginInfo));
                    renderPassInfo.framebuffer(swapchain.getFrameBuffers().get(i));

                    vkCmdBeginRenderPass(commandBuffer, renderPassInfo, VK_SUBPASS_CONTENTS_INLINE);

                    if (rosella.bufferManager != null && !simpleObjectManager.renderObjects.isEmpty()) {
                        bindBigBuffers(rosella.bufferManager, stack, commandBuffer);
                        for (RenderInfo renderInfo : simpleObjectManager.renderObjects.keySet()) {
                            for (InstanceInfo instance : simpleObjectManager.renderObjects.get(renderInfo)) {
                                bindInstanceInfo(instance, stack, commandBuffer, i); // TODO: check if the instance info from the previous one is the same
                                vkCmdDrawIndexed(
                                        commandBuffer,
                                        renderInfo.getIndicesSize(),
                                        1,
                                        rosella.bufferManager.indicesOffsetMap.getInt(renderInfo),
                                        rosella.bufferManager.vertexOffsetMap.getInt(renderInfo),
                                        0
                                );
                            }
                        }

                        vkCmdEndRenderPass(commandBuffer);
                        ok(vkEndCommandBuffer(commandBuffer));
                    }
                }
            }
        }
    }

    private void bindBigBuffers(GlobalBufferManager bufferManager, MemoryStack stack, VkCommandBuffer commandBuffer) {
        LongBuffer offsets = stack.longs(0);
        LongBuffer vertexBuffers = stack.longs(bufferManager.vertexBuffer.buffer());
        vkCmdBindVertexBuffers(commandBuffer, 0, vertexBuffers, offsets);
        vkCmdBindIndexBuffer(commandBuffer, bufferManager.indexBuffer.buffer(), 0, VK_INDEX_TYPE_UINT32);
    }

    private void bindInstanceInfo(InstanceInfo instanceInfo, MemoryStack stack, VkCommandBuffer commandBuffer, int commandBufferIndex) {
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
                stack.longs(instanceInfo.ubo().getDescriptors().getRawDescriptorSets().getLong(commandBufferIndex)),
                null
        );
    }

    // Stolen from https://github.com/SaschaWillems/Vulkan/blob/master/examples/screenshot/screenshot.cpp#L188
    // MIT license requires attribution
    public void screenshot(int width, int height) {
        VkDevice device = common.device.rawDevice;

        try (MemoryStack stack = MemoryStack.stackPush()) {
            // Check blit support for source and destination
            boolean useBlit;

            {
                VkFormatProperties properties = VkFormatProperties.mallocStack(stack);

                // Check if the device supports blitting from optimal images to linear images
                vkGetPhysicalDeviceFormatProperties(common.device.physicalDevice, swapchain.getSwapChainImageFormat(), properties);

                if ((properties.optimalTilingFeatures() & VK_FORMAT_FEATURE_BLIT_DST_BIT) != 0) {
                    vkGetPhysicalDeviceFormatProperties(common.device.physicalDevice, VK_FORMAT_R8G8B8A8_UNORM, properties);
                    useBlit = (properties.linearTilingFeatures() & VK_FORMAT_FEATURE_BLIT_DST_BIT) != 0;
                } else {
                    useBlit = false;
                }
            }

            // Source for the copy is the last rendered swapchain image
            // TODO: Get the last one fully rendered
            long srcImage = swapchain.getSwapChainImages().get(0);

            // Create the linear tiled destination image to copy to and to read the memory from
            VkImageCreateInfo info = VkImageCreateInfo.mallocStack(stack)
                    .sType(VK_IMAGE_TYPE_2D)
                    // Note that vkCmdBlitImage (if supported) will also do format conversions if the swapchain color format would differ
                    .format(VK_FORMAT_R8G8B8A8_UNORM)
                    .extent(extent -> extent.set(width, height, 1))
                    .arrayLayers(1)
                    .mipLevels(1)
                    .initialLayout(VK_IMAGE_LAYOUT_UNDEFINED)
                    .samples(VK_SAMPLE_COUNT_1_BIT)
                    .tiling(VK_IMAGE_TILING_LINEAR)
                    .usage(VK_IMAGE_USAGE_TRANSFER_DST_BIT);

            // Create the image
            BufferInfo destImage;

            {
                VmaAllocationCreateInfo flags = VmaAllocationCreateInfo.mallocStack(stack)
                        .flags(VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT);
                destImage = common.memory.createImageBuffer(info, flags);
            }

            // Do the actual blit from the swapchain image to our host visible destination image
            VkCommandBuffer commandBuffer;

            {
                PointerBuffer temp = stack.mallocPointer(1);
                VkCommandBufferAllocateInfo allocInfo = VkCommandBufferAllocateInfo.mallocStack(stack)
                        .sType(VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO)
                        .level(VK_COMMAND_BUFFER_LEVEL_PRIMARY)
                        .commandPool(commandPool)
                        .commandBufferCount(1);
                ok(vkAllocateCommandBuffers(device, allocInfo, temp));
                commandBuffer = new VkCommandBuffer(temp.get(), device);
                VkCommandBufferBeginInfo beginInfo = VkCommandBufferBeginInfo.mallocStack(stack)
                        .sType(VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO);
                ok(vkBeginCommandBuffer(commandBuffer, beginInfo));
            }

            // Transition destination image to transfer destination layout
            insertImageMemoryBarrier(commandBuffer,
                    destImage.buffer(),
                    0,
                    VK_ACCESS_TRANSFER_WRITE_BIT,
                    VK_IMAGE_LAYOUT_UNDEFINED,
                    VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
                    VK_PIPELINE_STAGE_TRANSFER_BIT,
                    VK_PIPELINE_STAGE_TRANSFER_BIT,
                    VkImageSubresourceRange.mallocStack(stack).set(VK_IMAGE_ASPECT_COLOR_BIT, 0, 1, 0, 1));
            // Transition swapchain image from present to transfer source layout
            insertImageMemoryBarrier(
                    commandBuffer,
                    srcImage,
                    VK_ACCESS_MEMORY_READ_BIT,
                    VK_ACCESS_TRANSFER_READ_BIT,
                    VK_IMAGE_LAYOUT_PRESENT_SRC_KHR,
                    VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL,
                    VK_PIPELINE_STAGE_TRANSFER_BIT,
                    VK_PIPELINE_STAGE_TRANSFER_BIT,
                    VkImageSubresourceRange.mallocStack(stack).set(VK_IMAGE_ASPECT_COLOR_BIT, 0, 1, 0, 1));

            // If source and destination support blit we'll blit as this also does automatic format conversion (e.g. from BGR to RGB)
            if (useBlit) {
                VkOffset3D size = VkOffset3D.mallocStack().set(width, height, 1);
                VkImageBlit.Buffer region = VkImageBlit.mallocStack(1);
                region.get()
                        .srcSubresource(srcSubresource -> {
                            srcSubresource.aspectMask(VK_IMAGE_ASPECT_COLOR_BIT);
                            srcSubresource.layerCount(1);
                        })
                        .dstSubresource(dstSubresource -> {
                            dstSubresource.aspectMask(VK_IMAGE_ASPECT_COLOR_BIT);
                            dstSubresource.layerCount(1);
                        });
                region.srcOffsets().put(1, size);
                region.dstOffsets().put(1, size);

                // Issue the copy command
                vkCmdBlitImage(
                        commandBuffer,
                        srcImage,
                        VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL,
                        destImage.buffer(),
                        VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
                        region,
                        VK_FILTER_NEAREST);
            } else {
                // Otherwise use image copy (requires us to manually flip components)
                VkImageCopy.Buffer region = VkImageCopy.mallocStack(1);
                region.get()
                        .srcSubresource(srcSubresource -> {
                            srcSubresource.aspectMask(VK_IMAGE_ASPECT_COLOR_BIT);
                            srcSubresource.layerCount(1);
                        })
                        .dstSubresource(dstSubresource -> {
                            dstSubresource.aspectMask(VK_IMAGE_ASPECT_COLOR_BIT);
                            dstSubresource.layerCount(1);
                        })
                        .extent(extent -> extent.set(width, height, 1));

                // Issue the copy command
                vkCmdCopyImage(
                        commandBuffer,
                        srcImage,
                        VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL,
                        destImage.buffer(),
                        VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
                        region);
            }

            // Transition destination image to general layout, which is the required layout for mapping the image memory later on
            insertImageMemoryBarrier(
                    commandBuffer,
                    destImage.buffer(),
                    VK_ACCESS_TRANSFER_WRITE_BIT,
                    VK_ACCESS_MEMORY_READ_BIT,
                    VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
                    VK_IMAGE_LAYOUT_GENERAL,
                    VK_PIPELINE_STAGE_TRANSFER_BIT,
                    VK_PIPELINE_STAGE_TRANSFER_BIT,
                    VkImageSubresourceRange.mallocStack(stack).set(VK_IMAGE_ASPECT_COLOR_BIT, 0, 1, 0, 1));

            // Transition back the swap chain image after the blit is done
            insertImageMemoryBarrier(
                    commandBuffer,
                    srcImage,
                    VK_ACCESS_TRANSFER_READ_BIT,
                    VK_ACCESS_MEMORY_READ_BIT,
                    VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL,
                    VK_IMAGE_LAYOUT_PRESENT_SRC_KHR,
                    VK_PIPELINE_STAGE_TRANSFER_BIT,
                    VK_PIPELINE_STAGE_TRANSFER_BIT,
                    VkImageSubresourceRange.mallocStack(stack).set(VK_IMAGE_ASPECT_COLOR_BIT, 0, 1, 0, 1));

            // Flush the command buffer
            ok(vkEndCommandBuffer(commandBuffer));

            {
                VkSubmitInfo submitInfo = VkSubmitInfo.mallocStack()
                        .sType(VK_STRUCTURE_TYPE_SUBMIT_INFO)
                        .pCommandBuffers(MemoryStack.stackPointers(commandBuffer));
                VkFenceCreateInfo fenceInfo = VkFenceCreateInfo.mallocStack()
                        .sType(VK_STRUCTURE_TYPE_FENCE_CREATE_INFO)
                        .flags(0);
                long fence;

                {
                    LongBuffer temp = MemoryStack.stackLongs(0);
                    ok(vkCreateFence(device, fenceInfo, null, temp));
                    fence = temp.get();
                }

                ok(vkQueueSubmit(queues.presentQueue.getQueue(), submitInfo, fence));
                ok(vkWaitForFences(device, fence, true, TimeUnit.SECONDS.toNanos(1)));
                vkDestroyFence(device, fence, null);
                vkFreeCommandBuffers(device, commandPool, commandBuffer);
            }

            // Get layout of the image (including row pitch)
            VkImageSubresource subresource = VkImageSubresource.callocStack(stack)
                    .set(VK_IMAGE_ASPECT_COLOR_BIT, 0, 0);
            VkSubresourceLayout layout = VkSubresourceLayout.callocStack(stack);
            vkGetImageSubresourceLayout(device, destImage.buffer(), subresource, layout);
            int offset = (int) layout.offset();
            int pitch = (int) layout.rowPitch();

            // Map image memory so we can start copying from it
            LongBuffer data;

            {
                PointerBuffer ppData = stack.mallocPointer(1);
                common.memory.map(destImage.allocation(), true, ppData);
                data = ppData.getLongBuffer(offset, pitch * height);
            }

            boolean isBGR = false;

            {
                int format = swapchain.getSwapChainImageFormat();
                isBGR |= format == VK_FORMAT_B8G8R8A8_SRGB;
                isBGR |= format == VK_FORMAT_B8G8R8A8_UNORM;
                isBGR |= format == VK_FORMAT_B8G8R8A8_SNORM;
            }

            // Copy image
            for (int y = 0; y < height; y++) {
                data.position(offset + y * pitch);

                for (int x = 0; x < width; x++) {
                    if (isBGR) {

                    } else {

                    }
                }
            }

            // Clean up resources
            common.memory.freeBuffer(destImage);
        }
    }

    private void insertImageMemoryBarrier(VkCommandBuffer commandBuffer, long destImage, int srcAccessMask, int dstAccessMask, int oldImageLayout, int newImageLayout, int srcStageMask, int dstStageMask, VkImageSubresourceRange subresourceRange) {
        VkImageMemoryBarrier.Buffer buffer = VkImageMemoryBarrier.malloc(1);
        buffer.get().set(VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER, MemoryUtil.NULL, srcAccessMask, dstAccessMask, oldImageLayout, newImageLayout, VK_QUEUE_FAMILY_IGNORED, VK_QUEUE_FAMILY_IGNORED, destImage, subresourceRange);

        vkCmdPipelineBarrier(
                commandBuffer,
                srcStageMask,
                dstStageMask,
                0,
                null,
                null,
                buffer);
    }

    public void clearColor(Color color) {
        if (clearColor != color) {
            lazilyClearColor(color);
            rebuildCommandBuffers(renderPass, ((SimpleObjectManager) rosella.objectManager));
        }
    }

    /**
     * Same as clearColor but you have to rebuild the command buffers
     *
     * @param color the colour you want the clear colour to change to
     */
    public void lazilyClearColor(Color color) {
        clearColor = color;
    }

    public void lazilyClearDepth(float depth) {
        clearDepth = depth;
    }

    public void lazilyClearStencil(int stencil) {
        clearStencil = stencil;
    }

    public void free() {
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
