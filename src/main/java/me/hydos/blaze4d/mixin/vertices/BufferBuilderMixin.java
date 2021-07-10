package me.hydos.blaze4d.mixin.vertices;

import com.google.common.collect.ImmutableList;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import me.hydos.blaze4d.api.vertex.ConsumerCreationInfo;
import me.hydos.blaze4d.api.vertex.UploadableConsumer;
import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.vertex.BufferVertexConsumer;
import me.hydos.rosella.render.vertex.VertexFormatElements;
import me.hydos.rosella.render.vertex.VertexFormats;
import net.minecraft.client.render.*;
import net.minecraft.util.math.Vec3f;
import org.joml.Matrix4f;
import org.joml.Vector3f;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.util.ArrayList;
import java.util.List;

import static net.minecraft.client.render.VertexFormats.*;

@Mixin(BufferBuilder.class)
public abstract class BufferBuilderMixin extends FixedColorVertexConsumer implements UploadableConsumer {

    private BufferVertexConsumer consumer;

    @Unique
    private static me.hydos.rosella.render.vertex.VertexFormatElement convertVertexFormatElement(VertexFormatElement mcElement) {
        if (mcElement.equals(POSITION_ELEMENT)) {
            return VertexFormatElements.POSITION;
        } else if (mcElement.equals(COLOR_ELEMENT)) {
            return VertexFormatElements.COLOR4ub;
        } else if (mcElement.equals(LIGHT_ELEMENT)) {
            return VertexFormatElements.UVs;
        } else if (mcElement.equals(NORMAL_ELEMENT)) {
            return VertexFormatElements.NORMAL;
        } else if (mcElement.equals(OVERLAY_ELEMENT)) {
            return VertexFormatElements.UVs;
        } else if (mcElement.equals(TEXTURE_0_ELEMENT)) {
            return VertexFormatElements.UVf;
        } else {
            throw new RuntimeException("IMPLEMENT CUSTOM VERTEX FORMAT ELEMENTS");
        }
    }

    @Inject(method = "begin", at = @At("HEAD"))
    private void setupConsumer(VertexFormat.DrawMode drawMode, VertexFormat format, CallbackInfo ci) {
        Matrix4f projMatrix = new Matrix4f(GlobalRenderSystem.projectionMatrix);
        Matrix4f viewMatrix = new Matrix4f(GlobalRenderSystem.modelViewMatrix);
        Vector3f chunkOffset = new Vector3f(GlobalRenderSystem.chunkOffset);
        Vec3f shaderLightDirections0 = GlobalRenderSystem.shaderLightDirections0.copy();
        Vec3f shaderLightDirections1 = GlobalRenderSystem.shaderLightDirections1.copy();

        this.consumer = GlobalRenderSystem.GLOBAL_CONSUMERS.computeIfAbsent(new ConsumerCreationInfo(drawMode, format, GlobalRenderSystem.createTextureArray(), GlobalRenderSystem.activeShader, projMatrix, viewMatrix, chunkOffset, shaderLightDirections0, shaderLightDirections1), consumerCreationInfo -> {
            BufferVertexConsumer consumer;
//            if (consumerCreationInfo.format().equals(POSITION)) {
//                consumer = new BufferVertexConsumer(VertexFormats.POSITION);
//            } else if (consumerCreationInfo.format().equals(POSITION_COLOR)) {
//                consumer = new BufferVertexConsumer(VertexFormats.POSITION_COLOR4);
//            } else if (consumerCreationInfo.format().equals(POSITION_COLOR_TEXTURE) || consumerCreationInfo.format().equals(BLIT_SCREEN)) {
//                consumer = new BufferVertexConsumer(VertexFormats.POSITION_COLOR4_UV);
//            } else if (consumerCreationInfo.format().equals(POSITION_TEXTURE)) {
//                consumer = new BufferVertexConsumer(VertexFormats.POSITION_UV);
//            } else if (consumerCreationInfo.format().equals(POSITION_TEXTURE_COLOR)) {
//                consumer = new BufferVertexConsumer(VertexFormats.POSITION_UV_COLOR4);
//            } else if (consumerCreationInfo.format().equals(LINES)) {
//                consumer = new BufferVertexConsumer(VertexFormats.POSITION_COLOR4_NORMAL);
//            } else if (consumerCreationInfo.format().equals(POSITION_COLOR_TEXTURE_LIGHT)) {
//                consumer = new BufferVertexConsumer(VertexFormats.POSITION_COLOR4_UV_LIGHT);
//            } else if (consumerCreationInfo.format().equals(POSITION_COLOR_TEXTURE_LIGHT_NORMAL)) {
//                consumer = new BufferVertexConsumer(VertexFormats.POSITION_COLOR4_UV_LIGHT_NORMAL);
//            } else if (consumerCreationInfo.format().equals(POSITION_COLOR_TEXTURE_OVERLAY_LIGHT_NORMAL)) {
//                consumer = new BufferVertexConsumer(VertexFormats.POSITION_COLOR4_UV_UV0_LIGHT_NORMAL);
//            } else if (consumerCreationInfo.format().equals(POSITION_TEXTURE_COLOR_NORMAL)) {
//                consumer = new BufferVertexConsumer(VertexFormats.POSITION_UV_COLOR4_NORMAL);
//            } else if (consumerCreationInfo.format().equals(POSITION_TEXTURE_COLOR_LIGHT)) {
//                consumer = new BufferVertexConsumer(VertexFormats.POSITION_UV_COLOR4_LIGHT);
//            } else {
            ImmutableList<VertexFormatElement> mcElements = consumerCreationInfo.format().getElements();
            List<me.hydos.rosella.render.vertex.VertexFormatElement> elementList = new ArrayList<>(mcElements.size()); // this size may change so we're not using a raw array
            for (VertexFormatElement mcElement : mcElements) {
                if (mcElement != null && !mcElement.equals(PADDING_ELEMENT)) { //FIXME: burger, the below thing adds null when padding is there but without padding for some reason the sky colour stays black unless padding is there padding somehow fixes the sky
                    elementList.add(convertVertexFormatElement(mcElement));
                }
            }
            consumer = new BufferVertexConsumer(VertexFormats.getFormat(elementList.toArray(me.hydos.rosella.render.vertex.VertexFormatElement[]::new)));
//            }
            return consumer;
        });
    }

    @Inject(method = "clear", at = @At("HEAD"))
    private void clear(CallbackInfo ci) {
    }

    @Override
    public VertexConsumer vertex(double x, double y, double z) {
        consumer.pos((float) x, (float) y, (float) z);
        return this;
    }

    @Override
    public VertexConsumer normal(float x, float y, float z) {
        consumer.normal(x, y, z);
        return this;
    }

    @Override
    public VertexConsumer color(int red, int green, int blue, int alpha) {
        consumer.color((byte) red, (byte) green, (byte) blue, (byte) alpha);
        return this;
    }

    @Override
    public VertexConsumer texture(float u, float v) {
        consumer.uv(u, v);
        return this;
    }

    @Override
    public VertexConsumer light(int u, int v) {
        consumer.light((short) u, (short) v);
        return this;
    }

    @Override
    public VertexConsumer overlay(int u, int v) {
        consumer.uv((short) u, (short) v);
        return this;
    }

    @Override
    public void vertex(float x, float y, float z, float red, float green, float blue, float alpha, float u, float v, int overlay, int light, float normalX, float normalY, float normalZ) {
        if (consumer.getFormat() == VertexFormats.POSITION_UV_COLOR4) {
            this.vertex(x, y, z);
            this.texture(u, v);
            this.color(red, green, blue, alpha);
            return;
        }

        if (consumer.getFormat() == VertexFormats.POSITION_UV) {
            this.vertex(x, y, z);
            this.texture(u, v);
            return;
        }

        this.vertex(x, y, z);
        this.color(red, green, blue, alpha);
        this.texture(u, v);
        if (consumer.getFormat() != VertexFormats.POSITION_COLOR4_UV_LIGHT_NORMAL) {
            this.overlay(overlay);
        }
        this.light(light);
        this.normal(normalX, normalY, normalZ);
        this.next();
    }

    /**
     * @author burgerdude
     * <p>
     * Redirect this function to our own consumer. This should really only be called
     * for custom attributes. We may want to end up putting all bytes through here.
     */
    @Overwrite
    public void putByte(int index, byte value) {
        consumer.putByte(index, value);
    }

    /**
     * @author burgerdude
     * <p>
     * read putShort
     */
    @Overwrite
    public void putShort(int index, short value) {
        consumer.putShort(index, value);
    }

    /**
     * @author burgerdude
     * <p>
     * read putFloat
     */
    @Overwrite
    public void putFloat(int index, float value) {
        consumer.putFloat(index, value);
    }

    @Override
    public void next() {
        consumer.nextVertex();
    }

    @Override
    public BufferVertexConsumer getConsumer() {
        return consumer;
    }

    @Override
    public ShaderProgram getShader() {
        return GlobalRenderSystem.activeShader;
    }
}
