package me.hydos.rosella.memory.dma;

import it.unimi.dsi.fastutil.longs.LongOpenHashSet;

import java.util.ArrayList;
import java.util.List;
import java.util.Set;

public class DMARecorder {

    private final Set<Long> acquiredBuffers = new LongOpenHashSet();
    private final Set<Long> releasedBuffers = new LongOpenHashSet();

    private final List<BufferAcquireTask> acquireTasks = new ArrayList<>();
    private final List<BufferReleaseTask> releaseTasks = new ArrayList<>();

    private final List<Long> waitSemaphores = new ArrayList<>();
    private final List<Long> signalSemaphores = new ArrayList<>();

    private int acquireBarrierCount = 0;
    private int releaseBarrierCount = 0;

    public DMARecorder() {
    }

    public void begin() {
    }

    public void end() {
    }

    public void reset() {
        acquiredBuffers.clear();
        releasedBuffers.clear();
        acquireTasks.clear();
        releaseTasks.clear();
        waitSemaphores.clear();
        signalSemaphores.clear();

        acquireBarrierCount = 0;
        releaseBarrierCount = 0;
    }
}
