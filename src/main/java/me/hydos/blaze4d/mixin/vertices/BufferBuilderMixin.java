package me.hydos.blaze4d.mixin.vertices;

import me.hydos.blaze4d.api.vertex.Blaze4dVertexStorage;
import net.minecraft.client.render.*;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.util.ArrayList;
import java.util.List;

@Mixin(BufferBuilder.class)
public abstract class BufferBuilderMixin extends FixedColorVertexConsumer implements Blaze4dVertexStorage, BufferVertexConsumer {

    @Shadow
    private VertexFormat format;

    private final me.hydos.rosella.render.vertex.BufferVertexConsumer consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(me.hydos.rosella.render.vertex.VertexFormats.Companion.getPOSITION_COLOR_UV());

    @Override
    public VertexConsumer vertex(double x, double y, double z) {
        consumer.pos((float) x, (float) y, (float) z);
        return this;
    }

    @Override
    public VertexConsumer color(int red, int green, int blue, int alpha) {
        consumer.color(red, green, blue);
        return this;
    }

    @Override
    public VertexConsumer texture(short u, short v, int index) {
        consumer.uv(u, v);
        return this;
    }

    @Override
    public void next() {
        consumer.nextVertex();
    }

    @Override
    public VertexFormat getVertexFormat() {
        return format;
    }

    @Override
    public me.hydos.rosella.render.vertex.BufferVertexConsumer getConsumer() {
        return consumer;
    }
}
