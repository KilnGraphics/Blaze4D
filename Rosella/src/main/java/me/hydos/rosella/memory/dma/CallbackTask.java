package me.hydos.rosella.memory.dma;

import me.hydos.rosella.Rosella;

public class CallbackTask extends Task {

    private final Runnable callback;

    public CallbackTask(Runnable callback) {
        this.callback = callback;
    }

    @Override
    public boolean scan(DMARecorder recorder) {
        return true;
    }

    @Override
    public void record(DMARecorder recorder) {
        recorder.addTask(this);
    }

    @Override
    public boolean shouldSignal() {
        return true;
    }

    @Override
    public void onCompleted() {
        this.callback.run();
    }
}
