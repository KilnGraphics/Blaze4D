package me.hydos.rosella.memory.allocators;

public interface DeviceAllocation {

    /**
     * @return The size of this allocation in bytes
     */
    long getByteSize();

    /**
     * Frees the memory that is backing this allocation
     */
    void free();
}
