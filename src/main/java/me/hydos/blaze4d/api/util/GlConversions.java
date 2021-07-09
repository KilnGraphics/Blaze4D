package me.hydos.blaze4d.api.util;

import me.hydos.rosella.render.texture.ImageFormat;
import org.lwjgl.opengl.GL11;
import org.lwjgl.opengl.GL14;
import org.lwjgl.opengl.GL30;
import org.lwjgl.opengl.GL33;
import org.lwjgl.vulkan.VK10;

public abstract class GlConversions {

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
}
