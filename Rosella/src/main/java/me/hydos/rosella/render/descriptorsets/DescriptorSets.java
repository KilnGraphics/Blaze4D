package me.hydos.rosella.render.descriptorsets;

import it.unimi.dsi.fastutil.longs.LongArrayList;
import me.hydos.rosella.device.VulkanDevice;
import me.hydos.rosella.memory.ManagedBuffer;
import me.hydos.rosella.memory.Memory;
import me.hydos.rosella.memory.MemoryCloseable;
import org.lwjgl.system.MemoryUtil;

import java.nio.LongBuffer;

public class DescriptorSets implements MemoryCloseable {

    private final LongArrayList descriptorSets;
    private long descriptorPool;

    public DescriptorSets(long descriptorPool, int initialSize) {
        this.descriptorSets = new LongArrayList(initialSize);
        this.descriptorPool = descriptorPool;
    }

    public DescriptorSets(long descriptorPool) {
        this(descriptorPool, 0);
    }

    @Override
    public void free(VulkanDevice device, Memory memory) {
        if (descriptorPool != 0L) {
            LongBuffer buffer = MemoryUtil.memAllocLong(descriptorSets.size());
            for (long descriptorSet : descriptorSets) {
                buffer.put(descriptorSet);
            }
            memory.freeDescriptorSets(descriptorPool, new ManagedBuffer<>(buffer, true));
        }
        descriptorSets.clear();
        // TODO: should we also set the descriptor pool to 0 here?
    }

    /**
     * Called after the descriptor pool has been freed, which frees the sets inside it.
     * We can let go of the pointers after this without worrying about freeing it ourselves.
     */
    public void clear() {
        descriptorSets.clear();
        descriptorPool = 0L;
    }

    public void add(long descriptorSet) {
        descriptorSets.add(descriptorSet);
    }

    public void setDescriptorPool(long descriptorPool) {
        this.descriptorPool = descriptorPool;
    }

    public LongArrayList getRawDescriptorSets() {
        return descriptorSets;
    }
}
