package me.hydos.blaze4d.mixin.integration;

import com.google.common.collect.ImmutableList;
import com.mojang.datafixers.util.Pair;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import me.hydos.blaze4d.api.util.ConversionUtils;
import me.hydos.blaze4d.api.vertex.ConsumerCreationInfo;
import me.hydos.blaze4d.mixin.vertices.BufferBuilderAccessor;
import me.hydos.rosella.render.vertex.StoredBufferProvider;
import me.hydos.rosella.render.vertex.VertexFormats;
import net.minecraft.client.render.BufferBuilder;
import net.minecraft.client.render.BufferRenderer;
import net.minecraft.client.render.VertexFormat;
import net.minecraft.client.render.VertexFormatElement;
import net.minecraft.util.math.Vec3f;
import org.joml.Matrix4f;
import org.joml.Vector3f;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.nio.ByteBuffer;

@Mixin(BufferRenderer.class)
public class BufferRendererMixin {

    @Inject(method = "draw(Lnet/minecraft/client/render/BufferBuilder;)V", at = @At("HEAD"), cancellable = true)
    private static void drawConsumer(BufferBuilder bufferBuilder, CallbackInfo ci) {
        Matrix4f projMatrix = new Matrix4f(GlobalRenderSystem.projectionMatrix);
        Matrix4f viewMatrix = new Matrix4f(GlobalRenderSystem.modelViewMatrix);
        Vector3f chunkOffset = new Vector3f(GlobalRenderSystem.chunkOffset);
        Vec3f shaderLightDirections0 = GlobalRenderSystem.shaderLightDirections0.copy();
        Vec3f shaderLightDirections1 = GlobalRenderSystem.shaderLightDirections1.copy();

        int srcBufferEnd = ((BufferBuilderAccessor) bufferBuilder).blaze4d$getNextDrawStart();

        Pair<BufferBuilder.DrawArrayParameters, ByteBuffer> drawData = bufferBuilder.popData();
        BufferBuilder.DrawArrayParameters drawInfo = drawData.getFirst(); // TODO: use the textured info from this to know if we should pass a blank texture array
        VertexFormat format = drawInfo.getVertexFormat();

        StoredBufferProvider storedBufferProvider = GlobalRenderSystem.GLOBAL_BUFFER_PROVIDERS.computeIfAbsent(new ConsumerCreationInfo(drawInfo.getMode(), format, GlobalRenderSystem.createTextureArray(), GlobalRenderSystem.activeShader, projMatrix, viewMatrix, chunkOffset, shaderLightDirections0, shaderLightDirections1), consumerCreationInfo -> {
            me.hydos.rosella.render.vertex.VertexFormat rosellaFormat = ConversionUtils.FORMAT_CONVERSION_MAP.get(consumerCreationInfo.format().getElements());

            if (rosellaFormat == null) {
                ImmutableList<VertexFormatElement> mcElements = consumerCreationInfo.format().getElements();
                me.hydos.rosella.render.vertex.VertexFormatElement[] rosellaElements = new me.hydos.rosella.render.vertex.VertexFormatElement[mcElements.size()]; // this size may change so we're not using a raw array
                for (int i = 0; i < mcElements.size(); i++) {
                    rosellaElements[i] = ConversionUtils.ELEMENT_CONVERSION_MAP.get(mcElements.get(i));
                }
                rosellaFormat = VertexFormats.getFormat(rosellaElements);
            }

            return new StoredBufferProvider(rosellaFormat);
        });

        int srcBufferStart = srcBufferEnd - (drawInfo.getCount() * format.getVertexSize());

        System.out.println(srcBufferStart + ", " + srcBufferEnd);

        storedBufferProvider.addBuffer(drawData.getSecond(), srcBufferStart, drawInfo.getCount()); // getCount is actually getVertexCount and someone mapped them wrong
        ci.cancel();
    }
}
