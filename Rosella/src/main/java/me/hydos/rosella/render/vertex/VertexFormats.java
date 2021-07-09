package me.hydos.rosella.render.vertex;

import it.unimi.dsi.fastutil.objects.Object2ObjectOpenHashMap;

import java.util.Map;

public class VertexFormats {

    private static final Map<VertexFormatElement[], VertexFormat> VERTEX_FORMAT_REGISTRY = new Object2ObjectOpenHashMap<>();

    public static final VertexFormat POSITION = getFormat(VertexFormatElements.POSITION);
    public static final VertexFormat POSITION_COLOR3 = getFormat(VertexFormatElements.POSITION, VertexFormatElements.COLOR3ub);
    public static final VertexFormat POSITION_COLOR3_UV = getFormat(VertexFormatElements.POSITION, VertexFormatElements.COLOR3ub, VertexFormatElements.UVf);
    public static final VertexFormat POSITION_UV = getFormat(VertexFormatElements.POSITION, VertexFormatElements.UVf);
    public static final VertexFormat POSITION_UV_COLOR3 = getFormat(VertexFormatElements.POSITION, VertexFormatElements.UVf, VertexFormatElements.COLOR3ub);
    public static final VertexFormat POSITION_COLOR4_NORMAL = getFormat(VertexFormatElements.POSITION, VertexFormatElements.COLOR4ub, VertexFormatElements.NORMAL);
    public static final VertexFormat POSITION_COLOR4_UV0_UV = getFormat(VertexFormatElements.POSITION, VertexFormatElements.COLOR4ub, VertexFormatElements.UVf, VertexFormatElements.UVs);
    public static final VertexFormat POSITION_COLOR4 = getFormat(VertexFormatElements.POSITION, VertexFormatElements.COLOR4ub);
    public static final VertexFormat POSITION_COLOR4_UV = getFormat(VertexFormatElements.POSITION, VertexFormatElements.COLOR4ub, VertexFormatElements.UVf);
    public static final VertexFormat POSITION_UV_COLOR4 = getFormat(VertexFormatElements.POSITION, VertexFormatElements.UVf, VertexFormatElements.COLOR4ub);
    public static final VertexFormat POSITION_UV_COLOR4_NORMAL = getFormat(VertexFormatElements.POSITION, VertexFormatElements.UVf, VertexFormatElements.COLOR4ub, VertexFormatElements.NORMAL);
    public static final VertexFormat POSITION_UV_COLOR4_LIGHT = getFormat(VertexFormatElements.POSITION, VertexFormatElements.UVf, VertexFormatElements.COLOR4ub, VertexFormatElements.UVs);
    public static final VertexFormat POSITION_COLOR4_UV_LIGHT = getFormat(VertexFormatElements.POSITION, VertexFormatElements.COLOR4ub, VertexFormatElements.UVf, VertexFormatElements.UVs);
    public static final VertexFormat POSITION_COLOR4_UV_LIGHT_NORMAL = getFormat(VertexFormatElements.POSITION, VertexFormatElements.COLOR4ub, VertexFormatElements.UVf, VertexFormatElements.UVs, VertexFormatElements.NORMAL);
    public static final VertexFormat POSITION_COLOR4_UV_UV0_LIGHT_NORMAL = getFormat(VertexFormatElements.POSITION, VertexFormatElements.COLOR4ub, VertexFormatElements.UVf, VertexFormatElements.UVs, VertexFormatElements.UVs, VertexFormatElements.NORMAL);

    // makes sure we don't waste a ton of memory with duplicates that get caught in the materials cache
    public static VertexFormat getFormat(VertexFormatElement... elements) {
        return VERTEX_FORMAT_REGISTRY.computeIfAbsent(elements, VertexFormat::new);
    }
}
