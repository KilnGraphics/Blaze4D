package me.hydos.blaze4d.mixin.vertices;

import me.hydos.blaze4d.api.VkRenderSystem;
import me.hydos.blaze4d.api.vertex.Blaze4dVertexStorage;
import me.hydos.rosella.render.shader.ShaderProgram;
import me.hydos.rosella.render.texture.UploadableImage;
import net.minecraft.client.MinecraftClient;
import net.minecraft.client.render.*;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(BufferBuilder.class)
public abstract class BufferBuilderMixin extends FixedColorVertexConsumer implements Blaze4dVertexStorage, BufferVertexConsumer {

    @Shadow
    private VertexFormat format;

    private me.hydos.rosella.render.vertex.BufferVertexConsumer consumer;
    private ShaderProgram shader;

    @Inject(method = "begin", at = @At("HEAD"))
    private void setupConsumer(VertexFormat.DrawMode drawMode, VertexFormat format, CallbackInfo ci) {
        this.shader = VkRenderSystem.activeShader;

        if (format == VertexFormats.POSITION) {
            consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(me.hydos.rosella.render.vertex.VertexFormats.Companion.getPOSITION());
        } else if (format == VertexFormats.POSITION_COLOR) {
            consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(me.hydos.rosella.render.vertex.VertexFormats.Companion.getPOSITION_COLOR());
        } else if (format == VertexFormats.POSITION_COLOR_TEXTURE) {
            consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(me.hydos.rosella.render.vertex.VertexFormats.Companion.getPOSITION_COLOR_UV());
        } else if (format == VertexFormats.POSITION_TEXTURE) {
            consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(me.hydos.rosella.render.vertex.VertexFormats.Companion.getPOSITION_UV());
        } else if (format == VertexFormats.POSITION_TEXTURE_COLOR) {
            consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(me.hydos.rosella.render.vertex.VertexFormats.Companion.getPOSITION_UV_COLOR());
        } else if (format == VertexFormats.LINES) {
            consumer = new me.hydos.rosella.render.vertex.BufferVertexConsumer(me.hydos.rosella.render.vertex.VertexFormats.Companion.getPOSITION_COLOR_NORMAL_PADDING());
        } else {
            throw new RuntimeException("Format not implemented: " + format);
        }
    }

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

    @Override
    public ShaderProgram getShader() {
        return shader;
    }

    @Override
    public UploadableImage getImage() {
        UploadableImage image = (UploadableImage) MinecraftClient.getInstance().getTextureManager().getTexture(VkRenderSystem.boundTexture);
        if(image == null) {
            throw new RuntimeException("Image is Null");
        }
        return image;
    }
}
