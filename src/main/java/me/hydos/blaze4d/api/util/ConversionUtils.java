package me.hydos.blaze4d.api.util;

import com.google.common.collect.ImmutableList;
import me.hydos.rosella.render.shader.ShaderType;
import me.hydos.rosella.render.texture.ImageFormat;
import me.hydos.rosella.render.vertex.VertexFormatElements;
import net.minecraft.client.gl.Program;
import net.minecraft.client.render.VertexFormat;
import net.minecraft.client.render.VertexFormatElement;
import net.minecraft.client.render.VertexFormats;
import org.lwjgl.opengl.GL11;
import org.lwjgl.opengl.GL14;
import org.lwjgl.opengl.GL30;
import org.lwjgl.opengl.GL33;
import org.lwjgl.vulkan.VK10;

import java.util.Collections;
import java.util.Map;

import static net.minecraft.client.render.VertexFormats.POSITION_ELEMENT;

public abstract class ConversionUtils {

    public static int glToVkBlendFunc(int glBlendFunc) {
        return switch (glBlendFunc) {
            case GL11.GL_ZERO -> VK10.VK_BLEND_FACTOR_ZERO;
            case GL11.GL_ONE -> VK10.VK_BLEND_FACTOR_ONE;
            case GL11.GL_SRC_COLOR -> VK10.VK_BLEND_FACTOR_SRC_COLOR;
            case GL11.GL_ONE_MINUS_SRC_COLOR -> VK10.VK_BLEND_FACTOR_ONE_MINUS_SRC_COLOR;
            case GL11.GL_DST_COLOR -> VK10.VK_BLEND_FACTOR_DST_COLOR;
            case GL11.GL_ONE_MINUS_DST_COLOR -> VK10.VK_BLEND_FACTOR_ONE_MINUS_DST_COLOR;
            case GL11.GL_SRC_ALPHA -> VK10.VK_BLEND_FACTOR_SRC_ALPHA;
            case GL11.GL_ONE_MINUS_SRC_ALPHA -> VK10.VK_BLEND_FACTOR_ONE_MINUS_SRC_ALPHA;
            case GL11.GL_DST_ALPHA -> VK10.VK_BLEND_FACTOR_DST_ALPHA;
            case GL11.GL_ONE_MINUS_DST_ALPHA -> VK10.VK_BLEND_FACTOR_ONE_MINUS_DST_ALPHA;
            case GL14.GL_CONSTANT_COLOR -> VK10.VK_BLEND_FACTOR_CONSTANT_COLOR;
            case GL14.GL_ONE_MINUS_CONSTANT_COLOR -> VK10.VK_BLEND_FACTOR_ONE_MINUS_CONSTANT_COLOR;
            case GL14.GL_CONSTANT_ALPHA -> VK10.VK_BLEND_FACTOR_CONSTANT_ALPHA;
            case GL14.GL_ONE_MINUS_CONSTANT_ALPHA -> VK10.VK_BLEND_FACTOR_ONE_MINUS_CONSTANT_ALPHA;
            case GL11.GL_SRC_ALPHA_SATURATE -> VK10.VK_BLEND_FACTOR_SRC_ALPHA_SATURATE;
            case GL33.GL_SRC1_COLOR -> VK10.VK_BLEND_FACTOR_SRC1_COLOR;
            case GL33.GL_ONE_MINUS_SRC1_COLOR -> VK10.VK_BLEND_FACTOR_ONE_MINUS_SRC1_COLOR;
            case GL33.GL_SRC1_ALPHA -> VK10.VK_BLEND_FACTOR_SRC1_ALPHA;
            case GL33.GL_ONE_MINUS_SRC1_ALPHA -> VK10.VK_BLEND_FACTOR_ONE_MINUS_SRC1_ALPHA;
            default -> throw new RuntimeException("GL blend func " + glBlendFunc + " is invalid or does not have a vulkan equivalent");
        };
    }

    public static int glToVkDepthFunc(int glDepthFunc) {
        return switch (glDepthFunc) {
            case GL11.GL_NEVER -> VK10.VK_COMPARE_OP_NEVER;
            case GL11.GL_LESS -> VK10.VK_COMPARE_OP_LESS;
            case GL11.GL_EQUAL -> VK10.VK_COMPARE_OP_EQUAL;
            case GL11.GL_LEQUAL -> VK10.VK_COMPARE_OP_LESS_OR_EQUAL;
            case GL11.GL_GREATER -> VK10.VK_COMPARE_OP_GREATER;
            case GL11.GL_NOTEQUAL -> VK10.VK_COMPARE_OP_NOT_EQUAL;
            case GL11.GL_GEQUAL -> VK10.VK_COMPARE_OP_GREATER_OR_EQUAL;
            case GL11.GL_ALWAYS -> VK10.VK_COMPARE_OP_ALWAYS;
            default -> throw new RuntimeException("GL depth func " + glDepthFunc + " is invalid or does not have a vulkan equivalent");
        };
    }

    public static int glToVkBlendOp(int glBlendOp) {
        return switch (glBlendOp) {
            case GL14.GL_FUNC_ADD -> VK10.VK_BLEND_OP_ADD;
            case GL14.GL_FUNC_SUBTRACT -> VK10.VK_BLEND_OP_SUBTRACT;
            case GL14.GL_FUNC_REVERSE_SUBTRACT -> VK10.VK_BLEND_OP_REVERSE_SUBTRACT;
            case GL14.GL_MIN -> VK10.VK_BLEND_OP_MIN;
            case GL14.GL_MAX -> VK10.VK_BLEND_OP_MAX;
            default -> throw new RuntimeException("GL blend op/equation " + glBlendOp + " is invalid or does not have a vulkan equivalent");
        };
    }

    public static int glToVkLogicOp(int glLogicOp) {
        return switch (glLogicOp) {
            case GL11.GL_CLEAR -> VK10.VK_LOGIC_OP_CLEAR;
            case GL11.GL_AND -> VK10.VK_LOGIC_OP_AND;
            case GL11.GL_AND_REVERSE -> VK10.VK_LOGIC_OP_AND_REVERSE;
            case GL11.GL_COPY -> VK10.VK_LOGIC_OP_COPY;
            case GL11.GL_AND_INVERTED -> VK10.VK_LOGIC_OP_AND_INVERTED;
            case GL11.GL_NOOP -> VK10.VK_LOGIC_OP_NO_OP;
            case GL11.GL_XOR -> VK10.VK_LOGIC_OP_XOR;
            case GL11.GL_OR -> VK10.VK_LOGIC_OP_OR;
            case GL11.GL_NOR -> VK10.VK_LOGIC_OP_NOR;
            case GL11.GL_EQUIV -> VK10.VK_LOGIC_OP_EQUIVALENT;
            case GL11.GL_INVERT -> VK10.VK_LOGIC_OP_INVERT;
            case GL11.GL_OR_REVERSE -> VK10.VK_LOGIC_OP_OR_REVERSE;
            case GL11.GL_COPY_INVERTED -> VK10.VK_LOGIC_OP_COPY_INVERTED;
            case GL11.GL_OR_INVERTED -> VK10.VK_LOGIC_OP_OR_INVERTED;
            case GL11.GL_NAND -> VK10.VK_LOGIC_OP_NAND;
            case GL11.GL_SET -> VK10.VK_LOGIC_OP_SET;
            default -> throw new RuntimeException("GL color logic op " + glLogicOp + " is invalid or does not have a vulkan equivalent");
        };
    }

    public static ImageFormat glToRosellaImageFormat(int glImageFormat) {
        return switch (glImageFormat) {
            case GL11.GL_RGBA -> ImageFormat.RGBA;
            case GL11.GL_RGB -> ImageFormat.RGB;
            case GL30.GL_RG -> ImageFormat.RG;
            case GL11.GL_RED -> ImageFormat.R;
            default -> throw new RuntimeException("GL image format " + glImageFormat + " is invalid or does not have a rosella equivalent");
        };
    }

    public static int glToVkDefaultImageFormat(int glImageFormat) {
        return switch (glImageFormat) {
            case GL11.GL_RGBA -> VK10.VK_FORMAT_R8G8B8A8_UNORM;
            case GL11.GL_RGB -> VK10.VK_FORMAT_R8G8B8_UNORM;
            case GL30.GL_RG -> VK10.VK_FORMAT_R8G8_UNORM;
            case GL11.GL_RED -> VK10.VK_FORMAT_R8_UNORM;
            default -> throw new RuntimeException("GL image format " + glImageFormat + " is invalid or does not have a vulkan equivalent");
        };
    }

    public static final Map<VertexFormatElement, me.hydos.rosella.render.vertex.VertexFormatElement> ELEMENT_CONVERSION_MAP = Map.of(
            VertexFormats.POSITION_ELEMENT, VertexFormatElements.POSITION,
            VertexFormats.COLOR_ELEMENT, VertexFormatElements.COLOR4ub,
            VertexFormats.LIGHT_ELEMENT, VertexFormatElements.UVs,
            VertexFormats.NORMAL_ELEMENT, VertexFormatElements.NORMAL,
            VertexFormats.OVERLAY_ELEMENT, VertexFormatElements.UVs,
            VertexFormats.TEXTURE_0_ELEMENT, VertexFormatElements.UVf,
            VertexFormats.PADDING_ELEMENT, VertexFormatElements.PADDINGb
    );

    // BLIT_SCREEN counts as a duplicate because the underlying list is the same as POSITION_TEXTURE_COLOR,
    // so it is not included here.
    // for some reason the default VertexFormat implementation has equals and hashcode not order dependant,
    // so we have to get the underlying element map and do it ourselves. this is still faster than manually
    // constructing a new one from the elements every time.
    public static final Map<ImmutableList<VertexFormatElement>, me.hydos.rosella.render.vertex.VertexFormat> FORMAT_CONVERSION_MAP = Map.ofEntries(
            Map.entry(VertexFormats.POSITION_COLOR_TEXTURE_LIGHT_NORMAL.getElements(), me.hydos.rosella.render.vertex.VertexFormats.POSITION_COLOR4_UV0_UV2_NORMAL),
            Map.entry(VertexFormats.POSITION_COLOR_TEXTURE_OVERLAY_LIGHT_NORMAL.getElements(), me.hydos.rosella.render.vertex.VertexFormats.POSITION_COLOR4_UV0_UV1_UV2_NORMAL),
            Map.entry(VertexFormats.POSITION_TEXTURE_COLOR_LIGHT.getElements(), me.hydos.rosella.render.vertex.VertexFormats.POSITION_UV0_COLOR4_UV2),
            Map.entry(VertexFormats.POSITION.getElements(), me.hydos.rosella.render.vertex.VertexFormats.POSITION),
            Map.entry(VertexFormats.POSITION_COLOR.getElements(), me.hydos.rosella.render.vertex.VertexFormats.POSITION_COLOR4),
            Map.entry(VertexFormats.LINES.getElements(), me.hydos.rosella.render.vertex.VertexFormats.POSITION_COLOR4_NORMAL),
            Map.entry(VertexFormats.POSITION_COLOR_LIGHT.getElements(), me.hydos.rosella.render.vertex.VertexFormats.POSITION_COLOR4_UV2),
            Map.entry(VertexFormats.POSITION_TEXTURE.getElements(), me.hydos.rosella.render.vertex.VertexFormats.POSITION_UV0),
            Map.entry(VertexFormats.POSITION_COLOR_TEXTURE.getElements(), me.hydos.rosella.render.vertex.VertexFormats.POSITION_COLOR4_UV0),
            Map.entry(VertexFormats.POSITION_TEXTURE_COLOR.getElements(), me.hydos.rosella.render.vertex.VertexFormats.POSITION_UV0_COLOR4),
            Map.entry(VertexFormats.POSITION_COLOR_TEXTURE_LIGHT.getElements(), me.hydos.rosella.render.vertex.VertexFormats.POSITION_COLOR4_UV0_UV2),
            Map.entry(VertexFormats.POSITION_TEXTURE_LIGHT_COLOR.getElements(), me.hydos.rosella.render.vertex.VertexFormats.POSITION_UV0_UV2_COLOR4),
            Map.entry(VertexFormats.POSITION_TEXTURE_COLOR_NORMAL.getElements(), me.hydos.rosella.render.vertex.VertexFormats.POSITION_UV0_COLOR4_NORMAL)
    );

    public static ShaderType mcToRosellaShaderType(Program.Type mcType) {
        return switch (mcType) {
            case VERTEX -> ShaderType.VERTEX_SHADER;
            case FRAGMENT -> ShaderType.FRAGMENT_SHADER;
        };
    }
}
