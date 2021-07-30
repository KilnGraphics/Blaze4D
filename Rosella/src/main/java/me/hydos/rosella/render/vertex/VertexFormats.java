package me.hydos.rosella.render.vertex;

import it.unimi.dsi.fastutil.Hash;
import it.unimi.dsi.fastutil.ints.Int2ObjectMap;
import it.unimi.dsi.fastutil.ints.Int2ObjectOpenHashMap;
import it.unimi.dsi.fastutil.objects.Object2ObjectMap;
import it.unimi.dsi.fastutil.objects.Object2ObjectOpenCustomHashMap;
import it.unimi.dsi.fastutil.objects.Object2ObjectOpenHashMap;

import java.util.Arrays;
import java.util.Objects;

public class VertexFormats {

    private static final Hash.Strategy<Object[]> ARRAY_HASH_STRATEGY = new Hash.Strategy<>() {
        @Override
        public int hashCode(Object[] o) {
            return Arrays.hashCode(o);
        }

        @Override
        public boolean equals(Object[] a, Object[] b) {
            return Arrays.equals(a, b);
        }
    };
    private static final Object2ObjectMap<VertexFormatElement[], VertexFormat> VERTEX_FORMAT_REGISTRY = new Object2ObjectOpenCustomHashMap<>(ARRAY_HASH_STRATEGY);

    // UV0 = TEXTURE, UV1 = OVERLAY, UV2 = LIGHT
    // ALL MC REQUIRED FORMATS:
    public static final VertexFormat POSITION_COLOR4_UV0_UV2_NORMAL = getFormat(VertexFormatElements.POSITION, VertexFormatElements.COLOR4ub, VertexFormatElements.UVf, VertexFormatElements.UVs, VertexFormatElements.NORMAL, VertexFormatElements.PADDINGb);
    public static final VertexFormat POSITION_COLOR4_UV0_UV1_UV2_NORMAL = getFormat(VertexFormatElements.POSITION, VertexFormatElements.COLOR4ub, VertexFormatElements.UVf, VertexFormatElements.UVs, VertexFormatElements.UVs, VertexFormatElements.NORMAL, VertexFormatElements.PADDINGb);
    public static final VertexFormat POSITION_UV0_COLOR4_UV2 = getFormat(VertexFormatElements.POSITION, VertexFormatElements.UVf, VertexFormatElements.COLOR4ub, VertexFormatElements.UVs);
    public static final VertexFormat POSITION = getFormat(VertexFormatElements.POSITION);
    public static final VertexFormat POSITION_COLOR4 = getFormat(VertexFormatElements.POSITION, VertexFormatElements.COLOR4ub);
    public static final VertexFormat POSITION_COLOR4_NORMAL = getFormat(VertexFormatElements.POSITION, VertexFormatElements.COLOR4ub, VertexFormatElements.NORMAL, VertexFormatElements.PADDINGb);
    public static final VertexFormat POSITION_COLOR4_UV2 = getFormat(VertexFormatElements.POSITION, VertexFormatElements.COLOR4ub, VertexFormatElements.UVs);
    public static final VertexFormat POSITION_UV0 = getFormat(VertexFormatElements.POSITION, VertexFormatElements.UVf);
    public static final VertexFormat POSITION_COLOR4_UV0 = getFormat(VertexFormatElements.POSITION, VertexFormatElements.COLOR4ub, VertexFormatElements.UVf);
    public static final VertexFormat POSITION_UV0_COLOR4 = getFormat(VertexFormatElements.POSITION, VertexFormatElements.UVf, VertexFormatElements.COLOR4ub);
    public static final VertexFormat POSITION_COLOR4_UV0_UV2 = getFormat(VertexFormatElements.POSITION, VertexFormatElements.COLOR4ub, VertexFormatElements.UVf, VertexFormatElements.UVs);
    public static final VertexFormat POSITION_UV0_UV2_COLOR4 = getFormat(VertexFormatElements.POSITION, VertexFormatElements.UVf, VertexFormatElements.UVs, VertexFormatElements.COLOR4ub);
    public static final VertexFormat POSITION_UV0_COLOR4_NORMAL = getFormat(VertexFormatElements.POSITION, VertexFormatElements.UVf, VertexFormatElements.COLOR4ub, VertexFormatElements.NORMAL, VertexFormatElements.PADDINGb);
    // ROSELLA & EXTRA FORMATS
    public static final VertexFormat POSITION_COLOR3f_UV0 = getFormat(VertexFormatElements.POSITION, VertexFormatElements.COLOR3f, VertexFormatElements.UVf);

    // makes sure we don't waste a ton of memory with duplicates that get caught in the materials cache
    public static VertexFormat getFormat(VertexFormatElement... elements) {
        return VERTEX_FORMAT_REGISTRY.computeIfAbsent(elements, i -> new VertexFormat(elements));
    }
}
