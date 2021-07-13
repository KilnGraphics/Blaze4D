package me.hydos.rosella.memory;

import org.jetbrains.annotations.Nullable;

import java.nio.ByteBuffer;

public class DMATransfer {

    /**
     * Returns the queue family used for transfer operations. Callers should use this to properly release / acquire
     * their resources.
     *
     * @return The transfer queue family index
     */
    public long getTransferQueueFamily() {
        return 0; // TODO: do not forget to actually update this once implemented
    }

    /**
     * Performs a buffer acquire operation in the transfer queue making the buffer available to transfer operations.
     * The release operation from the source queue and any memory barriers <b>must</b> first be performed by the callee.
     * If the <code>srcQueue</code> is equals to the transfer queue <b>no memory barrier will be inserted</b> by the transfer engine.
     *
     * @param buffer The buffer to acquire
     * @param srcQueue The queue that previously had ownership of the buffer
     * @param waitSemaphores A list of semaphores to wait on before executing the acquire operation
     * @param waitFences A list of fences to wait on before submitting the acquire operation
     */
    public void acquireBuffer(long buffer, long srcQueue, @Nullable long[] waitSemaphores, @Nullable long[] waitFences) {
    }

    /**
     * Performs a image acquire operation in the transfer queue making the image available to transfer operations.
     * The release operation from the source queue and any memory barriers <b>must</b> first be performed by the callee.
     * If the <code>srcQueue</code> is equals to the transfer queue <b>no memory barrier will be inserted</b> by the transfer engine.
     *
     * @param image The image to acquire
     * @param srcQueue The queue that previously had ownership of the image
     * @param waitSemaphores A list of semaphores to wait on before executing the acquire operation
     * @param waitFences A list of fences to wait on before submitting the acquire operation
     */
    public void acquireImage(long image, long srcQueue, @Nullable long[] waitSemaphores, @Nullable long[] waitFences) {
    }

    /**
     * Makes a shared buffer available to the transfer engine. This function does <b>not</b> perform any memory barrier
     * operations.
     *
     * @param buffer The buffer to acquire
     * @param waitSemaphores A list of semaphores to wait on before using the buffer
     * @param waitFences A list of fences to wait on before using the buffer
     */
    public void acquireSharedBuffer(long buffer, @Nullable long[] waitSemaphores, @Nullable long[] waitFences) {
    }

    /**
     * Makes a shared image available to the transfer engine. This function does <b>not</b> perform any memory barrier
     * operations.
     *
     * @param image The image to acquire
     * @param waitSemaphores A list of semaphores to wait on before using the image
     * @param waitFences A list of fences to wait on before using the image
     */
    public void acquireSharedImage(long image, @Nullable long[] waitSemaphores, @Nullable long[] waitFences) {
    }

    /**
     * Performs a buffer release operation on the transfer queue.
     * The acquire operation in the destination queue must be performed by the callee afterwards.
     * If the <code>dstQueue</code> is equals to the transfer queue <b>no memory barrier will be inserted</b> by the transfer engine.
     *
     * @param buffer The buffer to release
     * @param dstQueue The queue that will next take ownership of the buffer
     * @param signalSemaphores A list of semaphores to signal when the operation is complete
     * @param signalFences A list of fences to signal when the operation is complete
     */
    public void releaseBuffer(long buffer, long dstQueue, @Nullable long[] signalSemaphores, @Nullable long[] signalFences) {
    }

    /**
     * Performs a image release operation on the transfer queue.
     * The acquire operation in the destination queue must be performed by the callee afterwards.
     * If the <code>dstQueue</code> is equals to the transfer queue <b>no memory barrier will be inserted</b> by the transfer engine.
     *
     * @param image The image to release
     * @param dstQueue The queue that will next take ownership of the image
     * @param signalSemaphores A list of semaphores to signal when the operation is complete
     * @param signalFences A list of fences to signal when the operation is complete
     */
    public void releaseImage(long image, long dstQueue, @Nullable long[] signalSemaphores, @Nullable long[] signalFences) {
    }

    /**
     * Removes a shared buffer from the available buffers in the transfer engine. This function does <b>not</b> perform
     * any memory barrier operations.
     *
     * @param buffer The buffer to release
     * @param signalSemaphores A list of semaphores to signal when the buffer is ready to use
     * @param signalFences A list of fences to signal when the buffer is ready to use
     */
    public void releaseSharedBuffer(long buffer, @Nullable long[] signalSemaphores, @Nullable long[] signalFences) {
    }

    /**
     * Removes a shared image from the available images in the transfer engine. This function does <b>not</b> perform
     * any memory barrier operations.
     *
     * @param image The image to release
     * @param signalSemaphores A list of semaphores to signal when the image is ready to use
     * @param signalFences A list of fences to signal when the image is ready to use
     */
    public void releaseSharedImage(long image, @Nullable long[] signalSemaphores, @Nullable long[] signalFences) {
    }

    /**
     * Performs a transfer operation from host memory to a buffer. The entire source buffer range will be copied into
     * the destination buffer at the specified offset.
     * The destination buffer must first be made available to the transfer engine by calling any of the acquire functions.
     * The source buffer will be copied and can safely be overwritten after this function returns.
     *
     * @param srcBuffer The data to write into the destination buffer
     * @param dstBuffer The destination buffer
     * @param dstOffset The offset in the destination buffer to where the data should be copied to
     */
    public void transferBufferFromHost(ByteBuffer srcBuffer, long dstBuffer, long dstOffset) {
    }

    /**
     * Performs a transfer operation from a buffer to host memory. A region of the same length as the destination
     * buffer will be copied from the source buffer at the specified offset into the destination buffer.
     * The source buffer must first be made available to the transfer engine by calling any of the acquire functions.
     * The transfer engine will take ownership of the destination buffer until the operation is complete. The
     * destination buffer <b>must not</b> be modified until the operation has completed.
     *
     * @param srcBuffer The source buffer
     * @param srcOffset The offset in the source buffer from where the data should be copied from
     * @param dstBuffer The destination buffer
     * @param signalFences A list of fences to signal once the operation has completed
     */
    public void transferBufferToHost(long srcBuffer, long srcOffset, ByteBuffer dstBuffer, @Nullable long[] signalFences) {
    }


    // TODO: Images are pain
    public void transferImageFromHost(ByteBuffer srcBuffer, long dstImage, int dstImageLayout, BufferImageCopy copy) {
    }

    public void transferImageToHost(long srcImage, int srcImageLayout, ByteBuffer dstBuffer, BufferImageCopy copy) {
    }

    /**
     * Performs a transfer operation between 2 buffers.
     * Both buffers must first be made available to the transfer engine by calling any of the acquire functions.
     *
     * @param srcBuffer The source buffer
     * @param srcOffset The offset in the source buffer
     * @param dstBuffer The destination buffer
     * @param dstOffset The offset in the destination buffer
     * @param size The amount of data to copy
     */
    public void transferBuffer(long srcBuffer, long srcOffset, long dstBuffer, long dstOffset, long size) {
    }

    public record BufferImageCopy(long bufferOffset, int bufferRowLength, int bufferImageHeight) { // TODO: pain
    }
}
