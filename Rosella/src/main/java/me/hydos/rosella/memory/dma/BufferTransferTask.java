package me.hydos.rosella.memory.dma;

import me.hydos.rosella.Rosella;
import org.lwjgl.system.MemoryStack;

import javax.security.auth.login.CredentialNotFoundException;

public class BufferTransferTask extends Task {

    private final long srcBuffer;
    private final long dstBuffer;
    private final long srcOffset;
    private final long dstOffset;
    private final long size;

    public BufferTransferTask(long srcBuffer, long dstBuffer, long srcOffset, long dstOffset, long size) {
        this.srcBuffer = srcBuffer;
        this.dstBuffer = dstBuffer;
        this.srcOffset = srcOffset;
        this.dstOffset = dstOffset;
        this.size = size;
    }

    @Override
    public boolean scan(DMARecorder recorder) {
        if(recorder.hasWrittenBuffer(this.srcBuffer) || recorder.hasReadBuffer(this.dstBuffer)) {
            return false;
        }

        recorder.addReadBuffer(this.srcBuffer);
        recorder.addWriteBuffer(this.dstBuffer);
        return true;
    }

    @Override
    public void record(DMARecorder recorder) {
        recorder.recordBufferCopy(this.srcBuffer, this.dstBuffer, this.srcOffset, this.dstOffset, this.size);
    }

    @Override
    public boolean shouldSignal() {
        return false;
    }
}
