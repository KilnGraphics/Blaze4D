package me.hydos.blaze4d.mixin.buffers;

import com.mojang.blaze3d.platform.GlStateManager;
import com.mojang.blaze3d.systems.RenderSystem;
import org.jetbrains.annotations.Nullable;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;

import java.nio.ByteBuffer;

/**
 * Buffer Implementation just in case mods use it
 * TODO: this implementation can stay just in case, but i want to make IndexBuffer in RenderSystem work properly
 */
@Mixin(value = GlStateManager.class, remap = false)
public class GlStateManagerMixin {

    /**
     * @author Blaze4D
     * @reason to implement buffers
     */
    @Overwrite
    public static void _glBindBuffer(int target, int buffer) {
        RenderSystem.assertThread(RenderSystem::isOnRenderThreadOrInit);
//        GL15.glBindBuffer(target, buffer);
    }

    /**
     * @author Blaze4D
     * @reason to implement buffers
     */
    @Overwrite
    public static int _glGenBuffers() {
        RenderSystem.assertThread(RenderSystem::isOnRenderThreadOrInit);
        return 1;
//        return GL15.glGenBuffers();
    }

    /**
     * @author Blaze4D
     * @reason to implement buffers
     */
    @Overwrite
    public static void _glBufferData(int target, long size, int usage) {
        RenderSystem.assertThread(RenderSystem::isOnRenderThreadOrInit);
//        GL15.glBufferData(target, size, usage);
    }

    /**
     * @author Blaze4D
     * @reason to implement buffers
     */
    @Overwrite
    @Nullable
    public static ByteBuffer mapBuffer(int target, int access) {
        RenderSystem.assertThread(RenderSystem::isOnRenderThreadOrInit);
        return ByteBuffer.allocate(80092);
    }

    /**
     * @author Blaze4D
     * @reason to implement buffers
     */
    @Overwrite
    public static void _glUnmapBuffer(int target) {
        RenderSystem.assertThread(RenderSystem::isOnRenderThreadOrInit);
//        GL15.glUnmapBuffer(target);
    }
}
