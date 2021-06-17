package me.hydos.blaze4d.mixin.shader;

import com.mojang.blaze3d.platform.GlStateManager;
import com.mojang.blaze3d.systems.RenderSystem;
import it.unimi.dsi.fastutil.ints.Int2ObjectOpenHashMap;
import me.hydos.blaze4d.api.Blaze4dException;
import me.hydos.blaze4d.api.shader.ShaderContext;
import me.hydos.blaze4d.api.util.ByteArrayResource;
import me.hydos.rosella.render.resource.Resource;
import me.hydos.rosella.render.shader.RawShaderProgram;
import me.hydos.rosella.render.util.ShaderType;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;
import org.lwjgl.opengl.GL20;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Overwrite;

import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.Map;

/**
 * This Mixin handles the interactions between Minecraft shaders and GL programs and passes it onto rosella
 */
@Mixin(GlStateManager.class)
public class GlStateManagerMixin {

    private static final Logger LOGGER = LogManager.getLogger("Blaze4D VKStateManager");
    private static final Map<Integer, ShaderContext> SHADER_MAP = new Int2ObjectOpenHashMap<>();
    private static final Map<Integer, RawShaderProgram> SHADER_PROGRAM_MAP = new Int2ObjectOpenHashMap<>();
    private static final int DEFAULT_MAX_OBJECTS = 1024;
    private static String programErrorLog = "none";
    private static int nextShaderId = 0;
    private static int nextShaderProgramId = 1; // Minecraft is a special snowflake and needs shader program id's to start at 1

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
        ShaderType rosellaType = type == 35633 ? ShaderType.VERTEX_SHADER : ShaderType.FRAGMENT_SHADER;
        ShaderContext shaderContext = new ShaderContext();
        shaderContext.glShaderType = type;
        shaderContext.rosellaShaderType = rosellaType;
        SHADER_MAP.put(nextShaderId, shaderContext);
        nextShaderId++;
        return nextShaderId - 1;
    }

    /**
     * @author Blaze4D
     * @reason To Integrate Shaders
     */
    @Overwrite
    public static void glShaderSource(int shader, List<String> shaderLines) {
        RenderSystem.assertThread(RenderSystem::isOnRenderThread);
        ShaderContext context = SHADER_MAP.get(shader);
        if (context == null) {
            throw new Blaze4dException("Failed to get ShaderContext. (No shader was found with id " + shader + ")");
        }

        context.shader = shaderSrcToResource(shaderLines);
//        RawShaderProgram program = context.vulkanProgram;
//        if (program == null) {
//            program = new RawShaderProgram(null, null, Blaze4D.rosella.getDevice(), Blaze4D.rosella.getMemory(), DEFAULT_MAX_OBJECTS);
//        }
//
//        if (context.rosellaShaderType == ShaderType.VERTEX_SHADER) {
//            program.setVertexShader(shaderSrc);
//        } else {
//            program.setFragmentShader(shaderSrc);
//        }
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
        return 1;
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
        return nextShaderProgramId++;
    }

    /**
     * @author Blaze4D
     * @reason To Integrate Shader Programs
     */
    @Overwrite
    public static void _glBindAttribLocation(int program, int index, CharSequence name) {
        RenderSystem.assertThread(RenderSystem::isOnRenderThread);

        LOGGER.warn("Minecraft tried to bind to program " + program + " a attrib called " + name + " at id " + index);
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

            default -> programErrorLog = "glGetProgramI is not implemented for " + pname;
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
        String lastError = programErrorLog;
        programErrorLog = "";
        return lastError;
    }

    //========================
    //       UTILITIES
    //========================

    /**
     * Converts a list of lines of shader source code into a {@link Resource} which can be loaded by Rosella
     *
     * @param shaderSrc the source of the shader
     * @return a readable resource for {@link me.hydos.rosella.Rosella}
     */
    private static Resource shaderSrcToResource(List<String> shaderSrc) {
        StringBuilder src = new StringBuilder();
        for (String line : shaderSrc) {
            src.append(line);
            src.append("\n");
        }
        byte[] shaderBytes = src.toString().getBytes(StandardCharsets.UTF_8);
        return new ByteArrayResource(shaderBytes);
    }
}
