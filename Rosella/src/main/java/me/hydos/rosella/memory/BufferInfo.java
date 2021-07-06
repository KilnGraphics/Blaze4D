package me.hydos.rosella.memory;

import java.util.*;
import java.util.concurrent.ConcurrentHashMap;

import me.hydos.rosella.Rosella;
import me.hydos.rosella.device.VulkanDevice;

public record BufferInfo(long buffer, long allocation, RuntimeException stack) implements MemoryCloseable {
    public static final Map<BufferInfo, Integer> TIME_ALIVE = new ConcurrentHashMap<>();

    public BufferInfo(long buffer, long allocation, RuntimeException stack) {
        this.buffer = buffer;
        this.allocation = allocation;
        this.stack = stack;

        TIME_ALIVE.put(this, 1);
        updateCounts();
    }

    @Override
    public void free(VulkanDevice device, Memory memory) {
        memory.freeBuffer(this);
        synchronized (TIME_ALIVE) {
            TIME_ALIVE.remove(this);
        }
    }

    public synchronized static void updateCounts() {
        for (Map.Entry<BufferInfo, Integer> entry : TIME_ALIVE.entrySet()) {
            if (entry.getValue() > 10000) {
                Rosella.LOGGER.warn("Buffer alive for too long", entry.getKey().stack);
                TIME_ALIVE.remove(entry.getKey());
            } else {
                synchronized (TIME_ALIVE) {
                    if (TIME_ALIVE.containsKey(entry.getKey())) {
                        TIME_ALIVE.put(entry.getKey(), entry.getValue() + 1);
                    }
                }
            }
        }
    }
}
