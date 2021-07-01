package me.hydos.blaze4d.mixin.vertices;

import java.util.*;

import me.hydos.blaze4d.Blaze4D;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import me.hydos.blaze4d.api.vertex.ConsumerCreationInfo;
import me.hydos.blaze4d.api.vertex.ObjectInfo;
import me.hydos.blaze4d.api.vertex.UploadableConsumer;
import me.hydos.rosella.render.shader.ShaderProgram;
import org.joml.Matrix4f;
import org.joml.Vector3f;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import net.minecraft.client.render.*;
import net.minecraft.util.math.Vec3f;

@Mixin(BufferBuilder.class)
public abstract class BufferBuilderMixin extends FixedColorVertexConsumer implements UploadableConsumer {

    private Map<ConsumerCreationInfo, me.hydos.rosella.render.vertex.BufferVertexConsumer> consumers = new HashMap<>();
    private me.hydos.rosella.render.vertex.BufferVertexConsumer consumer;

    @Inject(method = "begin", at = @At("HEAD"))
    private void setupConsumer(VertexFormat.DrawMode drawMode, VertexFormat format, CallbackInfo ci) {
        Matrix4f projMatrix = copyMat4f(GlobalRenderSystem.projectionMatrix);
        Matrix4f viewMatrix = copyMat4f(GlobalRenderSystem.modelViewMatrix);
        Vector3f chunkOffset = copyVec3f(GlobalRenderSystem.chunkOffset);
        Vec3f shaderLightDirections0 = GlobalRenderSystem.shaderLightDirections0.copy();
        Vec3f shaderLightDirections1 = GlobalRenderSystem.shaderLightDirections1.copy();

        this.consumer = consumers.computeIfAbsent(new ConsumerCreationInfo(drawMode, format, format.getElements(), GlobalRenderSystem.boundTextureId, GlobalRenderSystem.activeShader, projMatrix, viewMatrix, chunkOffset, shaderLightDirections0, shaderLightDirections1), formats -> {
            me.hydos.rosella.render.vertex.BufferVertexConsumer consumer;
            if (format == VertexFormats.POSITION) {
                consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(me.hydos.rosella.render.vertex.VertexFormats.Companion.getPOSITION());
            } else if (format == VertexFormats.POSITION_COLOR) {
                consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(me.hydos.rosella.render.vertex.VertexFormats.Companion.getPOSITION_COLOR4());
            } else if (format == VertexFormats.POSITION_COLOR_TEXTURE) {
                consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(me.hydos.rosella.render.vertex.VertexFormats.Companion.getPOSITION_COLOR4_UV());
            } else if (format == VertexFormats.POSITION_TEXTURE) {
                consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(me.hydos.rosella.render.vertex.VertexFormats.Companion.getPOSITION_UV());
            } else if (format == VertexFormats.POSITION_TEXTURE_COLOR) {
                consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(me.hydos.rosella.render.vertex.VertexFormats.Companion.getPOSITION_UV_COLOR4());
            } else if (format == VertexFormats.LINES) {
                consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(me.hydos.rosella.render.vertex.VertexFormats.Companion.getPOSITION_COLOR_NORMAL());
            } else if (format == VertexFormats.POSITION_COLOR_TEXTURE_LIGHT) {
                consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(me.hydos.rosella.render.vertex.VertexFormats.Companion.getPOSITION_COLOR4_UV_LIGHT());
            } else if (format == VertexFormats.POSITION_COLOR_TEXTURE_LIGHT_NORMAL) {
                consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(me.hydos.rosella.render.vertex.VertexFormats.Companion.getPOSITION_COLOR4_UV_LIGHT_NORMAL());
            } else if (format == VertexFormats.POSITION_COLOR_TEXTURE_OVERLAY_LIGHT_NORMAL) {
                consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(me.hydos.rosella.render.vertex.VertexFormats.Companion.getPOSITION_COLOR4_UV_UV0_LIGHT_NORMAL());
            } else if (format == VertexFormats.POSITION_TEXTURE_COLOR_NORMAL) {
                consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(me.hydos.rosella.render.vertex.VertexFormats.Companion.getPOSITION_UV_COLOR4_NORMAL());
            } else if (format == VertexFormats.POSITION_TEXTURE_COLOR_LIGHT) {
                consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(me.hydos.rosella.render.vertex.VertexFormats.Companion.getPOSITION_UV_COLOR4_LIGHT());
            } else {
                // Check if its text
                List<VertexFormatElement> elements = format.getElements();
                if (elements.size() == 4 && elements.get(0) == VertexFormats.POSITION_ELEMENT && elements.get(1) == VertexFormats.COLOR_ELEMENT && elements.get(2) == VertexFormats.TEXTURE_0_ELEMENT && elements.get(3).getByteLength() == 4) {
                    consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(me.hydos.rosella.render.vertex.VertexFormats.Companion.getPOSITION_COLOR4_UV0_UV());
                } else {
                    throw new RuntimeException("Format not implemented: " + format);
                }
            }
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
        consumer.color(red, green, blue, alpha);
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
        if (consumer.getFormat() == me.hydos.rosella.render.vertex.VertexFormats.Companion.getPOSITION_UV_COLOR4()) {
            this.vertex(x, y, z);
            this.texture(u, v);
            this.color(red, green, blue, alpha);
            return;
        }
        this.vertex(x, y, z);
        this.color(red, green, blue, alpha);
        this.texture(u, v);
        if (consumer.getFormat() != me.hydos.rosella.render.vertex.VertexFormats.Companion.getPOSITION_COLOR4_UV_LIGHT_NORMAL()) {
            this.overlay(overlay);
        }
        this.light(light);
        this.normal(normalX, normalY, normalZ);
        this.next();
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

    @Override
    public int getTextureId() {
        return GlobalRenderSystem.boundTextureId;
    }

    @Override
    public void draw() {
        for (Map.Entry<ConsumerCreationInfo, me.hydos.rosella.render.vertex.BufferVertexConsumer> entry : consumers.entrySet()) {
            me.hydos.rosella.render.vertex.BufferVertexConsumer consumer = entry.getValue();
            List<Integer> indices = new ArrayList<>();
            ConsumerCreationInfo creationInfo = entry.getKey();

            if (creationInfo.drawMode() == VertexFormat.DrawMode.QUADS) {
                // Convert Quads to Triangle Strips
                //  0, 1, 2
                //  0, 2, 3
                //        v0_________________v1
                //         / \               /
                //        /     \           /
                //       /         \       /
                //      /             \   /
                //    v2-----------------v3

                for (int i = 0; i < consumer.getVertexCount(); i += 4) {
                    indices.add(i);
                    indices.add(1 + i);
                    indices.add(2 + i);

                    indices.add(2 + i);
                    indices.add(3 + i);
                    indices.add(i);
                }
            } else {
                for (int i = 0; i < consumer.getVertexCount(); i++) {
                    indices.add(i);
                }
            }

            if (consumer.getVertexCount() != 0) {
                ObjectInfo objectInfo = new ObjectInfo(
                        consumer,
                        creationInfo.drawMode(),
                        creationInfo.format(),
                        creationInfo.shader(),
                        creationInfo.boundTextureId(),
                        creationInfo.projMatrix(),
                        creationInfo.viewMatrix(),
                        creationInfo.chunkOffset(),
                        creationInfo.shaderLightDirections0(),
                        creationInfo.shaderLightDirections1(),
                        Collections.unmodifiableList(indices)
                );
                GlobalRenderSystem.uploadObject(objectInfo, Blaze4D.rosella);
            }
        }
        consumers.clear();
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
