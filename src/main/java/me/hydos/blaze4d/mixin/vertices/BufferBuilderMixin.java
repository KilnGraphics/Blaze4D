package me.hydos.blaze4d.mixin.vertices;

import com.google.common.collect.ImmutableList;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import me.hydos.blaze4d.api.vertex.ConsumerCreationInfo;
import me.hydos.blaze4d.api.vertex.UploadableConsumer;
import me.hydos.rosella.render.shader.ShaderProgram;
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

@Mixin(BufferBuilder.class)
public abstract class BufferBuilderMixin extends FixedColorVertexConsumer implements UploadableConsumer {

    private me.hydos.rosella.render.vertex.BufferVertexConsumer consumer;

    @Inject(method = "begin", at = @At("HEAD"))
    private void setupConsumer(VertexFormat.DrawMode drawMode, VertexFormat format, CallbackInfo ci) {
        Matrix4f projMatrix = copyMat4f(GlobalRenderSystem.projectionMatrix);
        Matrix4f viewMatrix = copyMat4f(GlobalRenderSystem.modelViewMatrix);
        Vector3f chunkOffset = copyVec3f(GlobalRenderSystem.chunkOffset);
        Vec3f shaderLightDirections0 = GlobalRenderSystem.shaderLightDirections0.copy();
        Vec3f shaderLightDirections1 = GlobalRenderSystem.shaderLightDirections1.copy();

        this.consumer = GlobalRenderSystem.GLOBAL_CONSUMERS_FOR_BATCH_RENDERING.computeIfAbsent(new ConsumerCreationInfo(drawMode, format, GlobalRenderSystem.createTextureArray(), GlobalRenderSystem.activeShader, projMatrix, viewMatrix, chunkOffset, shaderLightDirections0, shaderLightDirections1), consumerCreationInfo -> {
            me.hydos.rosella.render.vertex.BufferVertexConsumer consumer;
            if (consumerCreationInfo.format().equals(net.minecraft.client.render.VertexFormats.POSITION)) {
                consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(VertexFormats.POSITION);
            } else if (consumerCreationInfo.format().equals(net.minecraft.client.render.VertexFormats.POSITION_COLOR)) {
                consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(VertexFormats.POSITION_COLOR4);
            } else if (consumerCreationInfo.format().equals(net.minecraft.client.render.VertexFormats.POSITION_COLOR_TEXTURE) || consumerCreationInfo.format().equals(net.minecraft.client.render.VertexFormats.BLIT_SCREEN)) {
                consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(VertexFormats.POSITION_COLOR4_UV);
            } else if (consumerCreationInfo.format().equals(net.minecraft.client.render.VertexFormats.POSITION_TEXTURE)) {
                consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(VertexFormats.POSITION_UV);
            } else if (consumerCreationInfo.format().equals(net.minecraft.client.render.VertexFormats.POSITION_TEXTURE_COLOR)) {
                consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(VertexFormats.POSITION_UV_COLOR4);
            } else if (consumerCreationInfo.format().equals(net.minecraft.client.render.VertexFormats.LINES)) {
                consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(VertexFormats.POSITION_COLOR4_NORMAL);
            } else if (consumerCreationInfo.format().equals(net.minecraft.client.render.VertexFormats.POSITION_COLOR_TEXTURE_LIGHT)) {
                consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(VertexFormats.POSITION_COLOR4_UV_LIGHT);
            } else if (consumerCreationInfo.format().equals(net.minecraft.client.render.VertexFormats.POSITION_COLOR_TEXTURE_LIGHT_NORMAL)) {
                consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(VertexFormats.POSITION_COLOR4_UV_LIGHT_NORMAL);
            } else if (consumerCreationInfo.format().equals(net.minecraft.client.render.VertexFormats.POSITION_COLOR_TEXTURE_OVERLAY_LIGHT_NORMAL)) {
                consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(VertexFormats.POSITION_COLOR4_UV_UV0_LIGHT_NORMAL);
            } else if (consumerCreationInfo.format().equals(net.minecraft.client.render.VertexFormats.POSITION_TEXTURE_COLOR_NORMAL)) {
                consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(VertexFormats.POSITION_UV_COLOR4_NORMAL);
            } else if (consumerCreationInfo.format().equals(net.minecraft.client.render.VertexFormats.POSITION_TEXTURE_COLOR_LIGHT)) {
                consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(VertexFormats.POSITION_UV_COLOR4_LIGHT);
            } else {
                ImmutableList<VertexFormatElement> mcElements = consumerCreationInfo.format().getElements();
                List<me.hydos.rosella.render.vertex.VertexFormatElement> elementList = new ArrayList<>(mcElements.size()); // this size may change so we're not using a raw array
                for (VertexFormatElement mcElement : mcElements) {
                    elementList.add(convertVertexFormatElement(mcElement));
                }
                consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(VertexFormats.getFormat(elementList.toArray(me.hydos.rosella.render.vertex.VertexFormatElement[]::new)));
            }
            return consumer;
        });
   }

   @Unique
   private static me.hydos.rosella.render.vertex.VertexFormatElement convertVertexFormatElement(VertexFormatElement mcElement) {
        if (mcElement.equals(net.minecraft.client.render.VertexFormats.POSITION_ELEMENT)) {
            return VertexFormatElements.POSITION;
        } else if (mcElement.equals(net.minecraft.client.render.VertexFormats.COLOR_ELEMENT)) {
            return VertexFormatElements.COLOR4ub;
        } else if (mcElement.equals(net.minecraft.client.render.VertexFormats.LIGHT_ELEMENT)) {
            return VertexFormatElements.UVs;
        } else if (mcElement.equals(net.minecraft.client.render.VertexFormats.NORMAL_ELEMENT)) {
            return VertexFormatElements.NORMAL;
        } else if (mcElement.equals(net.minecraft.client.render.VertexFormats.OVERLAY_ELEMENT)) {
            return VertexFormatElements.UVs;
        } else if (mcElement.equals(net.minecraft.client.render.VertexFormats.TEXTURE_0_ELEMENT)) {
            return VertexFormatElements.UVf;
        } else if (mcElement.equals(net.minecraft.client.render.VertexFormats.PADDING_ELEMENT)) {
            return null;
        } else {
            throw new RuntimeException("IMPLEMENT CUSTOM VERTEX FORMAT ELEMENTS");
        }
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
     *
     * Redirect this function to our own consumer. This should really only be called
     * for custom attributes. We may want to end up putting all bytes thru here.
     */
    @Overwrite
    public void putByte(int index, byte value) {
        consumer.putByte(index, value);
    }

    /**
     * @author burgerdude
     *
     * read putByte
     */
    public void putShort(int index, short value) {
        consumer.putShort(index, value);
    }

    /**
     * @author burgerdude
     *
     * read putByte
     */
    public void putFloat(int index, float value) {
        consumer.putFloat(index, value);
    }

    @Override
    public void next() {
        consumer.nextVertex();
    }

    @Override
    public me.hydos.rosella.render.vertex.BufferVertexConsumer getConsumer() {
        return consumer;
    }

    @Override
    public ShaderProgram getShader() {
        return GlobalRenderSystem.activeShader;
    }

    protected Vector3f copyVec3f(Vector3f vec3f) {
        return new Vector3f(vec3f.x, vec3f.y, vec3f.z);
    }

    protected Matrix4f copyMat4f(Matrix4f mat4f) {
        Matrix4f newMatrix = new Matrix4f();
        newMatrix.m00(mat4f.m00());
        newMatrix.m01(mat4f.m01());
        newMatrix.m02(mat4f.m02());
        newMatrix.m03(mat4f.m03());

        newMatrix.m10(mat4f.m10());
        newMatrix.m11(mat4f.m11());
        newMatrix.m12(mat4f.m12());
        newMatrix.m13(mat4f.m13());

        newMatrix.m20(mat4f.m20());
        newMatrix.m21(mat4f.m21());
        newMatrix.m22(mat4f.m22());
        newMatrix.m23(mat4f.m23());

        newMatrix.m30(mat4f.m30());
        newMatrix.m31(mat4f.m31());
        newMatrix.m32(mat4f.m32());
        newMatrix.m33(mat4f.m33());

        return newMatrix;
    }
}
