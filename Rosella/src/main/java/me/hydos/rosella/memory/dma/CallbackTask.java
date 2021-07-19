package me.hydos.rosella.memory.dma;

public class CallbackTask extends Task {

    private final Runnable callback;

    public CallbackTask(Runnable callback) {
        this.callback = callback;
    }

    @Override
    public void record(DMARecorder recorder) {
        recorder.addCallback(callback);
    }
}
