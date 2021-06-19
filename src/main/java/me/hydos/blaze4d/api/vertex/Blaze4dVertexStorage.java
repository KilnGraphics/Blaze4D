package me.hydos.blaze4d.api.vertex;

import me.hydos.rosella.render.vertex.BufferVertexConsumer;
import net.minecraft.client.render.VertexFormat;

import java.util.List;

public interface Blaze4dVertexStorage {

    VertexFormat getVertexFormat();

    BufferVertexConsumer getConsumer();
}
