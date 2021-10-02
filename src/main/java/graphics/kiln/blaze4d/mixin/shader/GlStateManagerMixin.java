package graphics.kiln.blaze4d.mixin.shader;

import com.mojang.blaze3d.platform.GlStateManager;
import com.mojang.blaze3d.systems.RenderSystem;
import graphics.kiln.rosella.render.resource.Resource;
import graphics.kiln.rosella.render.shader.RawShaderProgram;
import graphics.kiln.rosella.render.shader.ShaderType;
import graphics.kiln.blaze4d.Blaze4D;
import graphics.kiln.blaze4d.impl.GlobalRenderSystem;
import graphics.kiln.blaze4d.impl.shader.MinecraftShaderProgram;
import graphics.kiln.blaze4d.api.shader.ShaderContext;
import graphics.kiln.blaze4d.util.ByteArrayResource;
import org.lwjgl.opengl.GL20;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;

import java.nio.FloatBuffer;
import java.nio.IntBuffer;
import java.nio.charset.StandardCharsets;
import java.util.List;

/**
 * This Mixin handles the interactions between Minecraft shaders and GL programs and passes it onto rosella
 */
@Mixin(value = GlStateManager.class, remap = false)
public class GlStateManagerMixin {

    //========================
    //        SHADERS
    //========================

    /**
     * @author Blaze4D
     * @reason To Integrate Shaders
     */
    @Overwrite
    public static int glCreateShader(int type) {
        RenderSystem.assertThread(RenderSystem::isOnRenderThread);
        // Check last shader's type to see if they belong in the same shader
        // TODO: maybe support more shader types in the future?
        ShaderType rosellaType = type == GL20.GL_VERTEX_SHADER ? ShaderType.VERTEX_SHADER : ShaderType.FRAGMENT_SHADER;
        ShaderContext shaderContext = new ShaderContext();
        shaderContext.glShaderType = type;
        shaderContext.rosellaShaderType = rosellaType;
        GlobalRenderSystem.SHADER_MAP.put(GlobalRenderSystem.nextShaderId, shaderContext);
        GlobalRenderSystem.nextShaderId++;
        return GlobalRenderSystem.nextShaderId - 1;
    }

    /**
     * @author Blaze4D
     * @reason To Integrate Shaders
     */
    @Overwrite
    public static void glShaderSource(int shader, List<String> shaderLines) {
        RenderSystem.assertThread(RenderSystem::isOnRenderThread);
        ShaderContext context = GlobalRenderSystem.SHADER_MAP.get(shader);
        if (context == null) {
            throw new RuntimeException("Failed to get ShaderContext. (No shader was found with id " + shader + ")");
        }

        context.shader = shaderSrcToResource(shaderLines);
    }

    /**
     * @author Blaze4D
     * @reason To Integrate Shaders
     * If something ever goes wrong, assume its our fault :(
     */
    @Overwrite
    public static String glGetShaderInfoLog(int shader, int maxLength) {
        RenderSystem.assertThread(RenderSystem::isOnRenderThread);
        return "Internal Blaze4D Error";
    }

    /**
     * @author Blaze4D
     * @reason To Integrate Shaders
     * <p>
     * This method is really just a method to get the compilation status of a shader.
     * as long as no exceptions have been thrown, assume everything is OK
     */
    @Overwrite
    public static int glGetShaderi(int shader, int pname) {
        RenderSystem.assertThread(RenderSystem::isOnRenderThread);
        return GL20.GL_TRUE;
    }

    //========================
    //    SHADER PROGRAMS
    //========================

    /**
     * @author Blaze4D
     * @reason To Integrate Shader Programs
     */
    @Overwrite
    public static int glCreateProgram() {
        RenderSystem.assertThread(RenderSystem::isOnRenderThread);
        MinecraftShaderProgram program = new MinecraftShaderProgram(
                null,
                null,
                Blaze4D.rosella.common.device,
                Blaze4D.rosella.common.memory,
                GlobalRenderSystem.DEFAULT_MAX_OBJECTS,
                GlobalRenderSystem.blaze4d$capturedShaderProgram.blaze4d$getUniforms(),
                GlobalRenderSystem.processedSamplers);
        GlobalRenderSystem.processedSamplers.clear();
        GlobalRenderSystem.currentSamplerBinding = 1;
        GlobalRenderSystem.SHADER_PROGRAM_MAP.put(GlobalRenderSystem.nextShaderProgramId, program);
        Blaze4D.rosella.renderer.rebuildCommandBuffers(Blaze4D.rosella.renderer.mainRenderPass);
        return GlobalRenderSystem.nextShaderProgramId++;
    }

    /**
     * @author Blaze4D
     * @reason To Integrate Shader Programs
     */
    @Overwrite
    public static void glAttachShader(int programId, int shaderId) {
        RenderSystem.assertThread(RenderSystem::isOnRenderThread);
        ShaderContext shader = GlobalRenderSystem.SHADER_MAP.get(shaderId);
        RawShaderProgram program = GlobalRenderSystem.SHADER_PROGRAM_MAP.get(programId);
        if (program == null) {
            throw new RuntimeException("Shader was requested without begin registered");
        }

        if (shader.rosellaShaderType == ShaderType.VERTEX_SHADER) {
            program.setVertexShader(shader.shader);
        } else {
            program.setFragmentShader(shader.shader);
        }
    }

    /**
     * @author Blaze4D
     * @reason To Integrate Shader Programs
     * <p>
     * Basically compiles the shader program
     */
    @Overwrite
    public static void glLinkProgram(int program) {
        RenderSystem.assertThread(RenderSystem::isOnRenderThread);
//        Identifier id = GlobalRenderSystem.generateId(program);
        Blaze4D.rosella.baseObjectManager.addShader(GlobalRenderSystem.SHADER_PROGRAM_MAP.get(program));
    }

    /**
     * @author Blaze4D
     * @reason To Integrate Shader Programs
     * <p>
     * Since shaders should define this in the vertex format, we shouldn't need to worry about this.
     */
    @Overwrite
    public static void _glBindAttribLocation(int program, int index, CharSequence name) {
        RenderSystem.assertThread(RenderSystem::isOnRenderThread);
    }

    /**
     * @author Blaze4D
     * @reason To Integrate Shader Programs
     * <p>
     * C Documentation: "Returns a parameter from a program object"
     * It really just's lets you query things from the program like status, etc
     */
    @Overwrite
    public static int glGetProgrami(int program, int pname) {
        RenderSystem.assertThread(RenderSystem::isOnRenderThread);
        switch (pname) {
            case GL20.GL_LINK_STATUS, GL20.GL_COMPILE_STATUS -> {
                // Since we throw exceptions instead of failing quietly, assume everything is OK
                return 1;
            }

            default -> GlobalRenderSystem.programErrorLog = "glGetProgramI is not implemented for " + pname;
        }
        return 0;
    }

    /**
     * @author Blaze4D
     * @reason To Integrate Shader Programs
     * <p>
     * When something errors, this is called to figure out what went wrong.
     */
    @Overwrite
    public static String glGetProgramInfoLog(int program, int maxLength) {
        RenderSystem.assertThread(RenderSystem::isOnRenderThread);
        String lastError = GlobalRenderSystem.programErrorLog;
        GlobalRenderSystem.programErrorLog = "";
        return lastError;
    }

    //========================
    //       UTILITIES
    //========================

    /**
     * Converts a list of lines of shader source code into a {@link Resource} which can be loaded by Rosella
     *
     * @param shaderSrc the source of the shader
     * @return a readable resource for {@link graphics.kiln.rosella.Rosella}
     */
    private static Resource shaderSrcToResource(List<String> shaderSrc) {
        byte[] shaderBytes = String.join("\n", shaderSrc).getBytes(StandardCharsets.UTF_8);
        return new ByteArrayResource(shaderBytes);
    }

    /**
     * @author Blaze4D
     */
    @Overwrite
    public static void glCompileShader(int shader) {
    }

    /**
     * @author Blaze4D
     */
    @Overwrite
    public static int _glGetUniformLocation(int program, CharSequence name) {
        return 0;
    }

    /**
     * @author Blaze4D
     */
    @Overwrite
    public static void glDeleteShader(int shader) {
    }

    /**
     * @author Blaze4D
     */
    @Overwrite
    public static void glDeleteProgram(int program) {
    }

    /**
     * @author Blaze4D
     */
    @Overwrite
    public static void _glUniform1(int location, IntBuffer value) {
    }

    /**
     * @author Blaze4D
     */
    @Overwrite
    public static void _glUniform1i(int location, int value) {
    }

    /**
     * @author Blaze4D
     */
    @Overwrite
    public static void _glUniform1(int location, FloatBuffer value) {
    }

    /**
     * @author Blaze4D
     */
    @Overwrite
    public static void _glUniform2(int location, IntBuffer value) {
    }

    /**
     * @author Blaze4D
     */
    @Overwrite
    public static void _glUniform2(int location, FloatBuffer value) {
    }

    /**
     * @author Blaze4D
     */
    @Overwrite
    public static void _glUniform3(int location, IntBuffer value) {
    }

    /**
     * @author Blaze4D
     */
    @Overwrite
    public static void _glUniform3(int location, FloatBuffer value) {
    }

    /**
     * @author Blaze4D
     */
    @Overwrite
    public static void _glUniform4(int location, IntBuffer value) {
    }

    /**
     * @author Blaze4D
     */
    @Overwrite
    public static void _glUniform4(int location, FloatBuffer value) {
    }

    /**
     * @author Blaze4D
     */
    @Overwrite
    public static void _glUniformMatrix2(int location, boolean transpose, FloatBuffer value) {
    }

    /**
     * @author Blaze4D
     */
    @Overwrite
    public static void _glUniformMatrix3(int location, boolean transpose, FloatBuffer value) {
    }

    /**
     * @author Blaze4D
     */
    @Overwrite
    public static void _glUniformMatrix4(int location, boolean transpose, FloatBuffer value) {
    }

    /**
     * @author Blaze4D
     */
    @Overwrite
    public static int _glGetAttribLocation(int program, CharSequence name) {
        return 0;
    }

    /**
     * @author Blaze4D
     */
    @Overwrite
    public static void _glUseProgram(int program) {
    }
}
