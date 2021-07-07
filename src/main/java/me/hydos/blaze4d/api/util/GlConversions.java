package me.hydos.blaze4d.api.util;

import org.lwjgl.opengl.*;
import org.lwjgl.vulkan.VK10;

public final class GlConversions {
    private GlConversions() {
        // noop
    }

    public static int glToVkBlendFunc(int glBlendFunc) {
        switch (glBlendFunc) {
            case GL11.GL_ZERO:
                return VK10.VK_BLEND_FACTOR_ZERO;
            case GL11.GL_ONE:
                return VK10.VK_BLEND_FACTOR_ONE;
            case GL11.GL_SRC_COLOR:
                return VK10.VK_BLEND_FACTOR_SRC_COLOR;
            case GL11.GL_ONE_MINUS_SRC_COLOR:
                return VK10.VK_BLEND_FACTOR_ONE_MINUS_SRC_COLOR;
            case GL11.GL_DST_COLOR:
                return VK10.VK_BLEND_FACTOR_DST_COLOR;
            case GL11.GL_ONE_MINUS_DST_COLOR:
                return VK10.VK_BLEND_FACTOR_ONE_MINUS_DST_COLOR;
            case GL11.GL_SRC_ALPHA:
                return VK10.VK_BLEND_FACTOR_SRC_ALPHA;
            case GL11.GL_ONE_MINUS_SRC_ALPHA:
                return VK10.VK_BLEND_FACTOR_ONE_MINUS_SRC_ALPHA;
            case GL11.GL_DST_ALPHA:
                return VK10.VK_BLEND_FACTOR_DST_ALPHA;
            case GL11.GL_ONE_MINUS_DST_ALPHA:
                return VK10.VK_BLEND_FACTOR_ONE_MINUS_DST_ALPHA;
            case GL14.GL_CONSTANT_COLOR:
                return VK10.VK_BLEND_FACTOR_CONSTANT_COLOR;
            case GL14.GL_ONE_MINUS_CONSTANT_COLOR:
                return VK10.VK_BLEND_FACTOR_ONE_MINUS_CONSTANT_COLOR;
            case GL14.GL_CONSTANT_ALPHA:
                return VK10.VK_BLEND_FACTOR_CONSTANT_ALPHA;
            case GL14.GL_ONE_MINUS_CONSTANT_ALPHA:
                return VK10.VK_BLEND_FACTOR_ONE_MINUS_CONSTANT_ALPHA;
            case GL11.GL_SRC_ALPHA_SATURATE:
                return VK10.VK_BLEND_FACTOR_SRC_ALPHA_SATURATE;
            case GL33.GL_SRC1_COLOR:
                return VK10.VK_BLEND_FACTOR_SRC1_COLOR;
            case GL33.GL_ONE_MINUS_SRC1_COLOR:
                return VK10.VK_BLEND_FACTOR_ONE_MINUS_SRC1_COLOR;
            case GL33.GL_SRC1_ALPHA:
                return VK10.VK_BLEND_FACTOR_SRC1_ALPHA;
            case GL33.GL_ONE_MINUS_SRC1_ALPHA:
                return VK10.VK_BLEND_FACTOR_ONE_MINUS_SRC1_ALPHA;
            default:
                throw new RuntimeException("GL blend func " + glBlendFunc + " is invalid or does not have a vulkan equivalent");
        }
    }

    public static int glToVkDepthFunc(int glDepthFunc) {
        switch (glDepthFunc) {
            case GL11.GL_NEVER:
                return VK10.VK_COMPARE_OP_NEVER;
            case GL11.GL_LESS:
                return VK10.VK_COMPARE_OP_LESS;
            case GL11.GL_EQUAL:
                return VK10.VK_COMPARE_OP_EQUAL;
            case GL11.GL_LEQUAL:
                return VK10.VK_COMPARE_OP_LESS_OR_EQUAL;
            case GL11.GL_GREATER:
                return VK10.VK_COMPARE_OP_GREATER;
            case GL11.GL_NOTEQUAL:
                return VK10.VK_COMPARE_OP_NOT_EQUAL;
            case GL11.GL_GEQUAL:
                return VK10.VK_COMPARE_OP_GREATER_OR_EQUAL;
            case GL11.GL_ALWAYS:
                return VK10.VK_COMPARE_OP_ALWAYS;
            default:
                throw new RuntimeException("GL depth func " + glDepthFunc + " is invalid or does not have a vulkan equivalent");
        }
    }

    public static int glToVkBlendOp(int glBlendOp) {
        switch (glBlendOp) {
            case GL14.GL_FUNC_ADD:
                return VK10.VK_BLEND_OP_ADD;
            case GL14.GL_FUNC_SUBTRACT:
                return VK10.VK_BLEND_OP_SUBTRACT;
            case GL14.GL_FUNC_REVERSE_SUBTRACT:
                return VK10.VK_BLEND_OP_REVERSE_SUBTRACT;
            case GL14.GL_MIN:
                return VK10.VK_BLEND_OP_MIN;
            case GL14.GL_MAX:
                return VK10.VK_BLEND_OP_MAX;
            default:
                throw new RuntimeException("GL blend op/equation " + glBlendOp + " is invalid or does not have a vulkan equivalent");
        }
    }

    public static int glToVkLogicOp(int glLogicOp) {
        switch (glLogicOp) {
            case GL11.GL_CLEAR:
                return VK10.VK_LOGIC_OP_CLEAR;
            case GL11.GL_AND:
                return VK10.VK_LOGIC_OP_AND;
            case GL11.GL_AND_REVERSE:
                return VK10.VK_LOGIC_OP_AND_REVERSE;
            case GL11.GL_COPY:
                return VK10.VK_LOGIC_OP_COPY;
            case GL11.GL_AND_INVERTED:
                return VK10.VK_LOGIC_OP_AND_INVERTED;
            case GL11.GL_NOOP:
                return VK10.VK_LOGIC_OP_NO_OP;
            case GL11.GL_XOR:
                return VK10.VK_LOGIC_OP_XOR;
            case GL11.GL_OR:
                return VK10.VK_LOGIC_OP_OR;
            case GL11.GL_NOR:
                return VK10.VK_LOGIC_OP_NOR;
            case GL11.GL_EQUIV:
                return VK10.VK_LOGIC_OP_EQUIVALENT;
            case GL11.GL_INVERT:
                return VK10.VK_LOGIC_OP_INVERT;
            case GL11.GL_OR_REVERSE:
                return VK10.VK_LOGIC_OP_OR_REVERSE;
            case GL11.GL_COPY_INVERTED:
                return VK10.VK_LOGIC_OP_COPY_INVERTED;
            case GL11.GL_OR_INVERTED:
                return VK10.VK_LOGIC_OP_OR_INVERTED;
            case GL11.GL_NAND:
                return VK10.VK_LOGIC_OP_NAND;
            case GL11.GL_SET:
                return VK10.VK_LOGIC_OP_SET;
            default:
                throw new RuntimeException("GL color logic op " + glLogicOp + " is invalid or does not have a vulkan equivalent");
        }
    }
}
