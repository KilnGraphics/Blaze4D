package me.hydos.blaze4d.api.shader;

import net.minecraft.util.Mth;

public interface VulkanUniform {
    void writeLocation(long address);

    int getMinecraftType();

    default int fixAlignment(int currentOffset) {
        return switch (getMinecraftType()) {
            case 1, 5 -> Mth.roundToward(currentOffset, 8);
            case 2, 3, 6, 7, 8, 9, 10 -> Mth.roundToward(currentOffset, 16);
            default -> currentOffset;
        };
    }
}
