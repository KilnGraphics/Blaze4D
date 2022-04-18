package graphics.kiln.blaze4d.mixin.render;

import com.mojang.blaze3d.vertex.BufferBuilder;
import com.mojang.blaze3d.vertex.BufferUploader;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;

@Mixin(BufferUploader.class)
public class BufferUploaderMixin {
//
//    /**
//     * @author Blaze4D
//     * @reason To draw Immediate Buffers
//     */
//    @Overwrite
//    public static void end(BufferBuilder builder) {
//        // Since this is immediate, there is no point storing the object. In the future, I might store 1 global BufferWrapper and have the methods take the VertexBuffer for context. That will be for a later date though.
//        new BasicImmediateBufferWrapper().render(builder);
//    }
}
