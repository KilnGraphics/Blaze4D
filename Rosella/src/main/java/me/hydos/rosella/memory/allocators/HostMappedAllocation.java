package me.hydos.rosella.memory.allocators;

import java.nio.ByteBuffer;

public interface HostMappedAllocation extends DeviceAllocation {

    /**
     * @return A buffer that writes to host mapped data
     */
    ByteBuffer getHostBuffer();
}
