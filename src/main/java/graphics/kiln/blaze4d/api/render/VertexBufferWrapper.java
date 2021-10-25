package graphics.kiln.blaze4d.api.render;

import com.mojang.blaze3d.vertex.BufferBuilder;
import graphics.kiln.rosella.render.shader.ShaderProgram;

import java.nio.ByteBuffer;

/**
 * Handles rendering a {@link com.mojang.blaze3d.vertex.VertexBuffer}
 */
public interface VertexBufferWrapper {

    /**
     * Called when Minecraft requests for the buffer to be uploaded
     *
     * @param bufferBuilder the BufferBuilder for the VertexBuffer the game wants uploaded.
     */
    void create(BufferBuilder bufferBuilder);

    /**
     * Called when Minecraft would usually call {@link com.mojang.blaze3d.systems.RenderSystem#drawElements(int, int, int)}
     */
    void render(ShaderProgram shaderProgram, ByteBuffer uboData);

    /**
     * Called when Minecraft is ready to close the {@link com.mojang.blaze3d.vertex.VertexBuffer}. You can assume past this point everything can be cleared
     */
    void clean();
}
