package me.hydos.rosella.memory.allocators;

import it.unimi.dsi.fastutil.longs.Long2ObjectLinkedOpenHashMap;

import java.util.Map;

public class UnbackedRingAllocator {
    private final long memorySize;

    private long virtualHead = 0;
    private long virtualTail = 0;

    private Map<Long, AllocationInfo> allocations = new Long2ObjectLinkedOpenHashMap<>();
    AllocationInfo lastAllocation = null;

    public UnbackedRingAllocator(long size) {
        this.memorySize = size;

        if(this.memorySize < 0) {
            throw new IllegalArgumentException("Invalid memory size " + size);
        }
        if(!isPowerOf2(this.memorySize)) {
            throw new IllegalArgumentException("Memory size must be power of 2 but was " + size);
        }
    }

    public boolean isEmpty() {
        return this.virtualHead == this.virtualTail;
    }

    public boolean isFull() {
        return this.virtualHead - this.virtualTail == this.memorySize;
    }

    public long allocate(final long size) {
        if(size <= 0) {
            throw new IllegalArgumentException("Size must be greater than 0 but was " + size);
        }

        final long alignedSize = (size + 7L) & -8L;

        long virtualFit = findVirtualFit(alignedSize);
        if(virtualFit == -1) {
            return -1;
        }

        if(virtualFit != this.virtualHead) {
            addAllocation(virtualFit, true);
        }
        addAllocation(virtualFit + size, false);

        return getRealOffset(virtualFit);
    }

    public void free(long address) {
        AllocationInfo info = allocations.remove(address);
        if(info == null) {
            throw new RuntimeException("Address passed to free is not a valid allocation");
        }

        info.setEmpty();
        if(address == getRealOffset(this.virtualTail)) {
            while(info != null && info.isEmpty()) {
                this.virtualTail = info.getEnd();
                info = info.getNext();
            }

            if(info == null) {
                this.lastAllocation = null;
                this.virtualHead = nextWraparound(this.virtualHead);
                this.virtualTail = this.virtualHead;
            }
        }
    }

    private void addAllocation(long end, boolean filler) {
        AllocationInfo info = new AllocationInfo(end);
        if(filler) {
            info.setEmpty();
        }

        allocations.put(getRealOffset(this.virtualHead), info);
        if(lastAllocation != null) {
            lastAllocation.setNext(info);
        }
        lastAllocation = info;

        this.virtualHead = end;
    }

    private long findVirtualFit(final long size) {
        if(this.isFull()) {
            return -1;
        }

        final long realHead = getRealOffset(this.virtualHead);
        final long realTail = getRealOffset(this.virtualTail);

        long result = -1;
        if(realHead < realTail) {
            if(realTail - realHead >= size) {
                result = this.virtualHead;
            }
        } else {
            if(this.memorySize - realHead >= size) {
                // There is enough space before a wraparound
                result = this.virtualHead;

            } else {
                if(realTail >= size) {
                    result = nextWraparound(this.virtualHead);
                }
            }
        }

        return result;
    }

    private long nextWraparound(final long virtualOffset) {
        return (virtualOffset + this.memorySize) & -this.memorySize;
    }

    private long getRealOffset(final long virtualOffset) {
        return virtualOffset & (this.memorySize - 1);
    }

    private static boolean isPowerOf2(long value) {
        return (value > 0) && ((value & (value-1)) == 0);
    }

    private static class AllocationInfo {
        private final long end;
        private AllocationInfo next = null;
        private boolean empty = false;

        public AllocationInfo(long end) {
            this.end = end;
        }

        public long getEnd() {
            return end;
        }

        public boolean isEmpty() {
            return empty;
        }

        public void setEmpty() {
            this.empty = true;
        }

        public void setNext(AllocationInfo next) {
            this.next = next;
        }

        public AllocationInfo getNext() {
            return this.next;
        }
    }
}
