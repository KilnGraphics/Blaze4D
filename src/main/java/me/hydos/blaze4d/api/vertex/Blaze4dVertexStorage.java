package me.hydos.blaze4d.api.vertex;

import net.minecraft.client.render.VertexFormat;

import java.util.List;

public interface Blaze4dVertexStorage {

    VertexFormat getVertexFormat();

    List<VertexData> getVertices();

    record VertexData (float x, float y, float z, int r, int g, int b){}
}
