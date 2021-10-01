package graphics.kiln.blaze4d.api.render;

import com.mojang.blaze3d.vertex.BufferBuilder;
import com.mojang.blaze3d.vertex.VertexBuffer;
import graphics.kiln.rosella.render.shader.ShaderProgram;

import java.nio.ByteBuffer;

/**
 * Handles rendering a {@link com.mojang.blaze3d.vertex.VertexBuffer}
 */
public interface VertexBufferWrapper {

    /**
     * Called when the game requests for the buffer to be uploaded
     * @param bufferBuilder the BufferBuilder for the VertexBuffer the game wants uploaded.
     */
    void create(BufferBuilder bufferBuilder);

    /**
     * Called when Minecraft would usually call {@link com.mojang.blaze3d.systems.RenderSystem#drawElements(int, int, int)}
     */
    void render(ShaderProgram shaderProgram, ByteBuffer uboData);

    void clean();
}
