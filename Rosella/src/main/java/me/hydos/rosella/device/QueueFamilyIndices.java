package me.hydos.rosella.device;

import java.util.Objects;

public class QueueFamilyIndices {

    public Integer graphicsFamily;
    public Integer presentFamily;

    public boolean isComplete() {
        return graphicsFamily != null && presentFamily != null;
    }

    public int[] unique() {
        if (Objects.equals(graphicsFamily, presentFamily)) {
            return new int[]{graphicsFamily};
        } else {
            return new int[]{graphicsFamily, presentFamily};
        }
    }
}
