package me.hydos.rosella.util;

import org.lwjgl.PointerBuffer;
import org.lwjgl.system.MemoryStack;

import java.util.List;
import java.util.Set;

/**
 * the new version of the old memory handler. will hopefully be much safer
 */
public class Memory {

    /**
     * Converts a {@link List} into a {@link PointerBuffer}
     *
     * @param list  the list to put into a {@link PointerBuffer}
     * @param stack the current {@link MemoryStack}
     * @return a valid {@link PointerBuffer}
     */
    public static PointerBuffer asPtrBuffer(List<String> list, MemoryStack stack) {
        PointerBuffer pBuffer = stack.mallocPointer(list.size());
        for (String object : list) {
            pBuffer.put(stack.UTF8Safe(object));
        }
        return pBuffer.rewind();
    }

    /**
     * Converts a {@link Set} into a {@link PointerBuffer}
     *
     * @param set  the list to put into a {@link PointerBuffer}
     * @param stack the current {@link MemoryStack}
     * @return a valid {@link PointerBuffer}
     */
    public static PointerBuffer asPtrBuffer(Set<String> set, MemoryStack stack) {
        PointerBuffer buffer = stack.mallocPointer(set.size());
        for (String object : set) {
            buffer.put(stack.UTF8(object));
        }

        return buffer.rewind();
    }
}
