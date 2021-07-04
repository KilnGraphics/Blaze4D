package me.hydos.rosella.device;

import java.util.stream.IntStream;

public class QueueFamilyIndices {
    public int graphicsFamily = -1;
    public int presentFamily = -1;

    public boolean isComplete() {
        return graphicsFamily != -1 && presentFamily != -1;
    }

    public int[] unique() {
        return IntStream.of(graphicsFamily, presentFamily).distinct().toArray();
    }
}
