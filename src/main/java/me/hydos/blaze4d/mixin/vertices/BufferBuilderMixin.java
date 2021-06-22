package me.hydos.blaze4d.mixin.vertices;

import me.hydos.blaze4d.api.GlobalRenderSystem;
import me.hydos.blaze4d.api.vertex.ConsumerRenderObject;
import me.hydos.blaze4d.api.vertex.UploadableConsumer;
import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.texture.UploadableImage;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.render.*;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.util.ArrayList;
import java.util.List;

@Mixin(BufferBuilder.class)
public abstract class BufferBuilderMixin extends FixedColorVertexConsumer implements UploadableConsumer {

    @Shadow
    private VertexFormat format;

    @Shadow
    private VertexFormat.DrawMode drawMode;

    private me.hydos.rosella.render.vertex.BufferVertexConsumer consumer;
    private ShaderProgram shader;

    @Inject(method = "begin", at = @At("HEAD"))
    private void setupConsumer(VertexFormat.DrawMode drawMode, VertexFormat format, CallbackInfo ci) {
        this.shader = GlobalRenderSystem.activeShader;

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
            consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(me.hydos.rosella.render.vertex.VertexFormats.Companion.getPOSITION_COLOR_NORMAL_PADDING());
        } else if (format == VertexFormats.POSITION_COLOR_TEXTURE_LIGHT) {
            consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(me.hydos.rosella.render.vertex.VertexFormats.Companion.getPOSITION_COLOR4_UV_LIGHT());
        } else {
            // Check if its text
            List<VertexFormatElement> elements = format.getElements();
            if (elements.size() == 4 && elements.get(0) == VertexFormats.POSITION_ELEMENT && elements.get(1) == VertexFormats.COLOR_ELEMENT && elements.get(2) == VertexFormats.TEXTURE_0_ELEMENT && elements.get(3).getByteLength() == 4) {
                consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(me.hydos.rosella.render.vertex.VertexFormats.Companion.getPOSITION_COLOR4_UV0_UV());
            } else {
                throw new RuntimeException("Format not implemented: " + format);
            }
        }
    }

    @Inject(method = "clear", at = @At("HEAD"))
    private void doCaching(CallbackInfo ci) {
        consumer.clear();
    }

    @Override
    public VertexConsumer vertex(double x, double y, double z) {
        consumer.pos((float) x, (float) y, (float) z);
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
    public VertexConsumer light(int light) {
        consumer.light(light);
        return this;
    }

    @Override
    public VertexConsumer overlay(int u, int v) {
        consumer.uv((short) u, (short) v);
        return this;
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
        return shader;
    }

    @Override
    public UploadableImage getImage() {
        UploadableImage image = (UploadableImage) MinecraftClient.getInstance().getTextureManager().getTexture(GlobalRenderSystem.boundTexture);
        if (image == null) {
            throw new RuntimeException("Image is Null");
        }
        return image;
    }

    @Override
    public void draw() {
        List<Integer> indices = new ArrayList<>();

        if (drawMode == VertexFormat.DrawMode.QUADS) {
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
            ConsumerRenderObject renderObject = new ConsumerRenderObject(
                    consumer.copy(),
                    drawMode,
                    format,
                    getShader(),
                    getImage()
            );
            renderObject.indices = indices;
            GlobalRenderSystem.uploadObject(renderObject);
        }
    }
}
