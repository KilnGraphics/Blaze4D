package me.hydos.rosella.memory;

import me.hydos.rosella.render.device.Device;
import me.hydos.rosella.render.util.memory.Memory;

/**
 * Used to safely close memory when an object wants to be de-allocated.
 */
public interface MemoryCloseable {

    void free(Device device, Memory memory);
}
