package me.hydos.rosella.debug;

import org.jetbrains.annotations.NotNull;
import org.lwjgl.vulkan.VK10;
import org.lwjgl.vulkan.VkDebugUtilsMessengerCallbackDataEXT;

import java.util.Objects;
import java.util.Set;
import java.util.concurrent.ConcurrentSkipListSet;
import java.util.concurrent.atomic.AtomicInteger;

public class VulkanDebugCallback {

    private final Set<Callback> callbacks = new ConcurrentSkipListSet<>();

    public VulkanDebugCallback() {
    }

    public void registerCallback(Callback cb) {
        this.callbacks.add(cb);
    }

    public void removeCallback(Callback cb) {
        this.callbacks.remove(cb);
    }

    public void destroy() {
        // The callback is registered during instance creation so we dont have to destroy anything
    }

    public int vulkanCallbackFunction(int severityBits, int typeBits, long pCallbackData, long pUserData) {
        try {
            VkDebugUtilsMessengerCallbackDataEXT callbackData = VkDebugUtilsMessengerCallbackDataEXT.create(pCallbackData);
            MessageSeverity severity = MessageSeverity.fromBits(severityBits);
            MessageType type = MessageType.fromBits(typeBits);

            callbacks.forEach(cb -> cb.call(severity, type, callbackData));

        } catch (Exception ex) {
            // TODO log?
        }
        return VK10.VK_FALSE;
    }

    public static abstract class Callback implements Comparable<Callback> {
        // Needed as a value to compare to for the skip list
        private static final AtomicInteger _nextId = new AtomicInteger(1);
        private final int _id;

        protected final AtomicInteger severityMask;
        protected final AtomicInteger typeMask;

        protected Callback(int severityMask, int typeMask) {
            this._id = _nextId.getAndIncrement();

            this.severityMask = new AtomicInteger(severityMask);
            this.typeMask = new AtomicInteger(typeMask);
        }

        public void call(MessageSeverity severity, MessageType type, VkDebugUtilsMessengerCallbackDataEXT data) {
            if(severity.isInMask(this.severityMask.get()) && type.isInMask(this.typeMask.get())) {
                this.callInternal(severity, type, data);
            }
        }

        /**
         * Function that is internally called if the defined severity and type filter matches.
         *
         * @param severity The severity of the message
         * @param type The type of the message
         * @param data The message
         */
        protected abstract void callInternal(MessageSeverity severity, MessageType type, VkDebugUtilsMessengerCallbackDataEXT data);

        @Override
        public final boolean equals(Object o) {
            if (this == o) return true;
            if (o == null || getClass() != o.getClass()) return false;
            Callback callback = (Callback) o;
            return _id == callback._id;
        }

        @Override
        public final int hashCode() {
            return Objects.hash(_id);
        }

        @Override
        public final int compareTo(@NotNull Callback other) {
            return this._id - other._id;
        }
    }
}
