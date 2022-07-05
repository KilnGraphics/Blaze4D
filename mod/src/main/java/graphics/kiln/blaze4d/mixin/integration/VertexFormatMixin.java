package graphics.kiln.blaze4d.mixin.integration;

import com.mojang.blaze3d.vertex.VertexFormat;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;

@Mixin(VertexFormat.IndexType.class)
public class VertexFormatMixin {
    @Overwrite
    public static VertexFormat.IndexType least(int i) {
        // We don't support byte indices
        // if ((i & 0xFFFF0000) != 0) {
        //     return VertexFormat.IndexType.INT;
        // }
        // return VertexFormat.IndexType.SHORT;
        return VertexFormat.IndexType.INT; // For now only ints
    }
}
