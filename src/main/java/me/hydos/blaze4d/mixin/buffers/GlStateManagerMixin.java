package me.hydos.blaze4d.mixin.buffers;

import com.mojang.blaze3d.platform.GlStateManager;
import com.mojang.blaze3d.systems.RenderSystem;
import it.unimi.dsi.fastutil.objects.Object2ObjectLinkedOpenHashMap;
import org.jetbrains.annotations.Nullable;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;

import java.nio.ByteBuffer;
import java.util.Map;

/**
 * Buffer Implementation just in case mods use it
 * TODO: this implementation can stay just in case, but i want to make IndexBuffer in RenderSystem work properly
 */
@Mixin(GlStateManager.class)
public class GlStateManagerMixin {

    private static final Map<Integer, ByteBuffer> BUFFER_MAP = new Object2ObjectLinkedOpenHashMap<>();
    private static int NEXT_BUFFER_ID = 1;

    /**
     * @author Blaze4D
     * @reason to implement buffers
     */
    @Overwrite
    public static void _glBindBuffer(int target, int buffer) {
        RenderSystem.assertThread(RenderSystem::isOnRenderThreadOrInit);
    }

    /**
     * @author Blaze4D
     * @reason to implement buffers
     */
    @Overwrite
    public static int _glGenBuffers() {
        RenderSystem.assertThread(RenderSystem::isOnRenderThreadOrInit);
        return NEXT_BUFFER_ID++;
    }

    /**
     * @author Blaze4D
     * @reason to implement buffers
     */
    @Overwrite
    public static void _glBufferData(int target, long size, int usage) {
        RenderSystem.assertThread(RenderSystem::isOnRenderThreadOrInit);
        BUFFER_MAP.put(target, ByteBuffer.allocate((int) size));
    }

    /**
     * @author Blaze4D
     * @reason to implement buffers
     */
    @Overwrite
    @Nullable
    public static ByteBuffer mapBuffer(int target, int access) {
        RenderSystem.assertThread(RenderSystem::isOnRenderThreadOrInit);
        ByteBuffer buffer = ByteBuffer.allocate(80092);
        BUFFER_MAP.put(target, buffer);
        return buffer;
    }

    /**
     * @author Blaze4D
     * @reason to implement buffers
     */
    @Overwrite
    public static void _glUnmapBuffer(int target) {
        RenderSystem.assertThread(RenderSystem::isOnRenderThreadOrInit);
        BUFFER_MAP.remove(target).clear();
    }
}
