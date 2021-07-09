package me.hydos.rosella.render.swapchain

import org.lwjgl.system.MemoryStack.stackGet
import java.nio.LongBuffer

/**
 * Represents a frame that will be rendered to the window
 */
class Frame(
    private val imageAvailableSemaphore: Long,
    private val renderFinishedSemaphore: Long,
    private val fence: Long
) {
    fun imageAvailableSemaphore(): Long {
        return imageAvailableSemaphore
    }

    fun pImageAvailableSemaphore(): LongBuffer {
        return stackGet().longs(imageAvailableSemaphore)
    }

    fun renderFinishedSemaphore(): Long {
        return renderFinishedSemaphore
    }

    fun pRenderFinishedSemaphore(): LongBuffer {
        return stackGet().longs(renderFinishedSemaphore)
    }

    fun fence(): Long {
        return fence
    }

    fun pFence(): LongBuffer {
        return stackGet().longs(fence)
    }
}
