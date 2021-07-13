package me.hydos.rosella.memory.dma;

import java.util.concurrent.atomic.AtomicBoolean;

public abstract class Task {

    private final AtomicBoolean ready;
    protected Task next = null;

    protected Task(boolean initialReady) {
        this.ready = new AtomicBoolean(initialReady);
    }

    public boolean isReady() {
        return this.ready.get();
    }

    public void setReady() {
        this.ready.set(true);
    }

    public void setNext(Task next) {
        this.next = next;
    }

    public Task getNext() {
        return this.next;
    }

    public abstract boolean shouldSignal();

    public void onCompleted() {
    }
}
