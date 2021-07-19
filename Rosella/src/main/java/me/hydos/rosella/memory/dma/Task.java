package me.hydos.rosella.memory.dma;

import java.util.concurrent.atomic.AtomicBoolean;

public abstract class Task {

    protected Task next = null;

    protected Task() {
    }

    public void setNext(Task next) {
        this.next = next;
    }

    public Task getNext() {
        return this.next;
    }

    public boolean canReorderBehind(Task other) {
        return false;
    }
    public Task tryMergeWith(Task other) {
        return null;
    }

    public boolean canRecord(DMARecorder recorder) {
        return true;
    }
    public abstract void record(DMARecorder recorder);
}
