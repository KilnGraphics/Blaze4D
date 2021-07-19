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

    public abstract boolean scan(DMARecorder recorder);
    public abstract void record(DMARecorder recorder);

    public abstract boolean shouldSignal();

    public void onCompleted() {
    }
}
