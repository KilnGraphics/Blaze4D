package me.hydos.blaze4d.api.vertex;

import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.texture.UploadableImage;
import me.hydos.rosella.render.vertex.BufferVertexConsumer;
import net.minecraft.client.render.VertexFormat;

import java.util.List;

public interface UploadableConsumer {

    BufferVertexConsumer getConsumer();

    ShaderProgram getShader();

    int getTextureId();

    void draw();
}
