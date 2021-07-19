package me.hydos.rosella.memory.dma;

import it.unimi.dsi.fastutil.longs.LongArraySet;

import java.util.Collection;
import java.util.Set;

public class SignalSemaphoreTask extends Task {

    private final Set<Long> signalSemaphores = new LongArraySet();

    public SignalSemaphoreTask(long signalSemaphore) {
        this.signalSemaphores.add(signalSemaphore);
    }

    public SignalSemaphoreTask(Collection<Long> signalSemaphores) {
        this.signalSemaphores.addAll(signalSemaphores);
    }

    @Override
    public boolean canRecord(DMARecorder recorder) {
        return true;
    }

    @Override
    public void record(DMARecorder recorder) {
        recorder.addSignalSemaphores(this.signalSemaphores);
    }
}
