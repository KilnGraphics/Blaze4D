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

    public List<VertexData> elements = new ArrayList<>();

    public double x = 0;
    public double y = 0;
    public double z = 0;

    public int r = 0;
    public int g = 0;
    public int b = 0;

    @Override
    public VertexConsumer vertex(double x, double y, double z) {
        this.x = x;
        this.y = y;
        this.z = z;
        return BufferVertexConsumer.super.vertex(x, y, z);
    }

    @Override
    public VertexConsumer color(int red, int green, int blue, int alpha) {
        this.r = red;
        this.g = green;
        this.b = blue;
        return BufferVertexConsumer.super.color(red, green, blue, alpha);
    }

    @Override
    public void next() {
        elements.add(new VertexData(
                (float) x, (float) y, (float) z,
                r, g, b
        ));
    }

    @Override
    public VertexFormat getVertexFormat() {
        return format;
    }

    @Override
    public List<VertexData> getVertices() {
        return elements;
    }
}
