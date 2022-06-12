package graphics.kiln.blaze4d.mixin.render;

import com.mojang.blaze3d.systems.RenderSystem;
import com.mojang.blaze3d.vertex.BufferBuilder;
import com.mojang.blaze3d.vertex.BufferUploader;
import graphics.kiln.blaze4d.Blaze4D;
import graphics.kiln.blaze4d.api.B4DShader;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

import java.util.Objects;

@Mixin(BufferUploader.class)
public class BufferUploaderMixin {

    @Inject(method = "drawWithShader", at = @At("HEAD"))
    private static void drawImmediate(BufferBuilder.RenderedBuffer renderedBuffer, CallbackInfo ci) {
        if (renderedBuffer.indexBuffer().remaining() != 0) {

            BufferBuilder.DrawState drawState = renderedBuffer.drawState();
        }
    }
}
