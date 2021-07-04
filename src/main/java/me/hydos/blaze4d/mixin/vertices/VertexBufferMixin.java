package me.hydos.blaze4d.mixin.vertices;

import me.hydos.blaze4d.api.GlobalRenderSystem;
import me.hydos.blaze4d.api.vertex.ConsumerCreationInfo;
import me.hydos.blaze4d.api.vertex.UploadableConsumer;
import me.hydos.rosella.render.vertex.BufferVertexConsumer;
import net.minecraft.client.gl.VertexBuffer;
import net.minecraft.client.render.BufferBuilder;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;

import java.util.Map;

/**
 * Turns out, Minecraft uses this class for world rendering. when a part of the world is to be rendered, the buffer will be cleared and replaced with just the sky. this will then be uploaded to a {@link VertexBuffer} and then cleared again for the rest of the game to render.
 */
@Mixin(VertexBuffer.class)
public class VertexBufferMixin {

    /**
     * @author Blaze4D
     * @reason To render the world
     */
    @Overwrite
    private void uploadInternal(BufferBuilder buffer) {
        Map<ConsumerCreationInfo, BufferVertexConsumer> consumers = ((UploadableConsumer) buffer).getConsumers();
        GlobalRenderSystem.renderConsumers(consumers);
    }
}
