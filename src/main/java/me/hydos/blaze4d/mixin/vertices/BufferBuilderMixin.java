package me.hydos.blaze4d.mixin.vertices;

import com.google.common.collect.ImmutableList;
import com.mojang.datafixers.util.Pair;
import me.hydos.blaze4d.api.GlobalRenderSystem;
import me.hydos.blaze4d.api.util.ConversionUtils;
import me.hydos.blaze4d.api.vertex.ConsumerCreationInfo;
import me.hydos.rosella.render.vertex.StoredBufferProvider;
import me.hydos.rosella.render.vertex.VertexFormats;
import net.minecraft.client.render.BufferBuilder;
import net.minecraft.client.render.FixedColorVertexConsumer;
import net.minecraft.client.render.VertexFormat;
import net.minecraft.client.render.VertexFormatElement;
import net.minecraft.util.math.Vec3f;
import org.joml.Matrix4f;
import org.joml.Vector3f;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.nio.ByteBuffer;

@Mixin(BufferBuilder.class)
public abstract class BufferBuilderMixin extends FixedColorVertexConsumer {

    @Shadow public abstract Pair<BufferBuilder.DrawArrayParameters, ByteBuffer> popData();

    @Inject(method = "end", at = @At("TAIL"))
    private void addToBufferStorage(CallbackInfo ci) {
        Matrix4f projMatrix = new Matrix4f(GlobalRenderSystem.projectionMatrix);
        Matrix4f viewMatrix = new Matrix4f(GlobalRenderSystem.modelViewMatrix);
        Vector3f chunkOffset = new Vector3f(GlobalRenderSystem.chunkOffset);
        Vec3f shaderLightDirections0 = GlobalRenderSystem.shaderLightDirections0.copy();
        Vec3f shaderLightDirections1 = GlobalRenderSystem.shaderLightDirections1.copy();

        Pair<BufferBuilder.DrawArrayParameters, ByteBuffer> drawData = this.popData();
        BufferBuilder.DrawArrayParameters drawInfo = drawData.getFirst(); // TODO: use the textured info from this to know if we should pass a blank texture array
        VertexFormat format = drawInfo.getVertexFormat();

        StoredBufferProvider storedBufferProvider = GlobalRenderSystem.GLOBAL_BUFFER_PROVIDERS.computeIfAbsent(new ConsumerCreationInfo(drawInfo.getMode(), format, GlobalRenderSystem.createTextureArray(), GlobalRenderSystem.activeShader, projMatrix, viewMatrix, chunkOffset, shaderLightDirections0, shaderLightDirections1), consumerCreationInfo -> {
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
            ImmutableList<VertexFormatElement> mcElements = format.getElements();
            me.hydos.rosella.render.vertex.VertexFormatElement[] rosellaElements = new me.hydos.rosella.render.vertex.VertexFormatElement[mcElements.size()]; // this size may change so we're not using a raw array
            for (int i = 0; i < mcElements.size(); i++) {
                rosellaElements[i] = ConversionUtils.ELEMENT_CONVERSION_MAP.get(mcElements.get(i));
            }
            return new StoredBufferProvider(VertexFormats.getFormat(rosellaElements));
        });

        storedBufferProvider.addBuffer(drawData.getSecond(), drawInfo.getCount()); // getCount is actually getVertexCount and someone mapped them wrong
    }

}
