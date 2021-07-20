package me.hydos.rosella.memory.dma;

import it.unimi.dsi.fastutil.longs.LongArraySet;

import java.util.Collection;
import java.util.Set;

public class WaitSemaphoreTask extends Task {

    private Set<Long> waitSemaphores = new LongArraySet();

    public WaitSemaphoreTask(long waitSemaphore) {
        this.waitSemaphores.add(waitSemaphore);
    }

    public WaitSemaphoreTask(Collection<Long> waitSemaphores) {
        this.waitSemaphores.addAll(waitSemaphores);
    }

    @Override
    public boolean canRecord(DMARecorder recorder) {
        return !recorder.containsSignalSemaphores();
    }

    @Override
    public void record(DMARecorder recorder) {
        recorder.addWaitSemaphores(this.waitSemaphores);
    }
}
