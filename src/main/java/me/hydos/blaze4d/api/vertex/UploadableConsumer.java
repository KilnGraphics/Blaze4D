package me.hydos.blaze4d.api.vertex;

import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.vertex.BufferVertexConsumer;

public interface UploadableConsumer {

    BufferVertexConsumer getConsumer();

    ShaderProgram getShader();

}
