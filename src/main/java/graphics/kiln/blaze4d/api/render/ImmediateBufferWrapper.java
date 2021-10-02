package graphics.kiln.blaze4d.api.render;

import com.mojang.blaze3d.vertex.BufferBuilder;

/**
 * Handles rendering anything built with {@link com.mojang.blaze3d.vertex.BufferBuilder} (A type of {@link com.mojang.blaze3d.vertex.VertexConsumer})and rendered with a {@link com.mojang.blaze3d.vertex.BufferUploader}
 */
public interface ImmediateBufferWrapper {

    /**
     * Since Minecraft uses immediate rendering for almost everything which isn't a chunk, We need to both upload and render at the same time. It isn't ideal and if we could, we would apply higher level patches to make it work better. Cleaning is done later on inside of Rosella and shouldn't need to be handled here
     */
    void render(BufferBuilder builder);
}
